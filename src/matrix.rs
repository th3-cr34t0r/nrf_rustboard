use crate::config::{
    COLS, ENTER_SLEEP_DEBOUNCE, KEY_DEBOUNCE, KEY_INTERUPT_DEBOUNCE, MATRIX_KEYS_BUFFER, ROWS,
};
use crate::keycodes::KC;
use crate::{LAYER, MATRIX_KEYS, delay_ms, delay_us};

use core::pin::pin;
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_time::{Duration, Instant};
use heapless::Vec;

#[cfg(feature = "debug")]
use defmt::info;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct KeyPos {
    pub row: u8,
    pub col: u8,
    pub layer: u8,
}

impl KeyPos {
    pub fn default() -> Self {
        Self {
            row: 255,
            col: 255,
            layer: 255,
        }
    }
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum KeyState {
    #[default]
    Released,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub code: KC,
    pub position: KeyPos,
    pub time: Instant,
    pub state: KeyState,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KC::EU,
            position: KeyPos::default(),
            time: Instant::now(),
            state: KeyState::default(),
        }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    reg_keys_local_new: [KeyPos; MATRIX_KEYS_BUFFER],
    // reg_keys_local_new: Vec<Key, MATRIX_KEYS_BUFFER>,
    // reg_keys_local_old: Vec<Key, MATRIX_KEYS_BUFFER>,
    reg_keys_local_written_time: Instant,
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            reg_keys_local_new: [KeyPos::default(); MATRIX_KEYS_BUFFER],
            // reg_keys_local_new: Vec::new(),
            // reg_keys_local_old: Vec::new(),
            reg_keys_local_written_time: Instant::now(),
        }
    }

    fn is_elapsed(time: &Instant, debounce: Duration) -> bool {
        if Instant::now() >= *time + debounce {
            true
        } else {
            false
        }
    }

    /// Main function for scanning and registering keys
    pub async fn scan(&mut self) {
        loop {
            if Self::is_elapsed(&self.reg_key_last_time, KEY_INTERUPT_DEBOUNCE) {
                for row in self.rows.iter_mut() {
                    row.set_high();
                    // delay so port propagates
                    delay_us(1).await;
                }

                // set cols wait for high
                {
                    let mut futures: Vec<_, COLS> = self
                        .cols
                        .iter_mut()
                        .map(|col| col.wait_for_high())
                        .collect();

                    match select(
                        select_slice(pin!(futures.as_mut_slice())),
                        delay_ms(ENTER_SLEEP_DEBOUNCE),
                    )
                    .await
                    {
                        Either::First(_) => {
                            // key has been pressed, scan it
                        }
                        Either::Second(()) => {
                            // enter sleep
                            // TODO:
                        }
                    }
                }
            }

            for (row_count, row) in self.rows.iter_mut().enumerate() {
                row.set_high();
                // delay so port propagates
                delay_us(1).await;

                // get the pressed keys
                for (col_count, col) in self.cols.iter().enumerate() {
                    if col.is_high() {
                        let new_key_position = KeyPos {
                            row: row_count as u8,
                            col: col_count as u8,
                            layer: *LAYER.lock().await,
                        };

                        if let Some(index) = self
                            .reg_keys_local_new
                            .iter()
                            .position(|&key_pos| key_pos == KeyPos::default())
                        {
                            self.reg_keys_local_new[index] = new_key_position;
                        };
                    }
                }

                // set row to low
                row.set_low();
            }

            // send reg_keys every 1 ms
            if Self::is_elapsed(&self.reg_keys_local_written_time, Duration::from_millis(1)) {
                let mut matrix_keys_locked = MATRIX_KEYS.lock().await;

                self.reg_keys_local_new.iter().for_each(|l_key| {
                    if l_key != KeyPos::default() {
                        if let Some(g_key_index) = matrix_keys_locked
                            .iter_mut()
                            .position(|g_key| g_key.position == l_key.position)
                        {
                            matrix_keys_locked[g_key_index].time = Instant::now();
                            matrix_keys_locked[g_key_index].state = KeyState::Pressed;
                        } else {
                            let key = Key {
                                code: KC::EU,
                                position: *l_key,
                                time: Instant::now(),
                                state: KeyState::Pressed,
                            };
                            matrix_keys_locked.push(key).expect(
                                "[matrix_scan] error pushing new key into global matrix_keys",
                            );
                        }

                        *l_key = KeyPos::default();
                    }
                });

                self.reg_keys_local_written_time = Instant::now();
            }
        }
    }
}
