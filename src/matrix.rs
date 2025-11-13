use crate::config::{
    COLS, ENTER_SLEEP_DEBOUNCE, KEY_DEBOUNCE, KEY_INTERUPT_DEBOUNCE, REGISTERED_KEYS_BUFFER, ROWS,
};
use crate::keycodes::KC;
use crate::{REGISTERED_KEYS, delay_ms, delay_us};

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
}

impl KeyPos {
    pub fn default() -> Self {
        Self { row: 255, col: 255 }
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
    reg_keys_local_new: Vec<Key, REGISTERED_KEYS_BUFFER>,
    reg_keys_local_old: Vec<Key, REGISTERED_KEYS_BUFFER>,
    reg_keys_local_written_time: Instant,
    reg_key_last_time: Instant,
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            reg_keys_local_new: Vec::new(),
            reg_keys_local_old: Vec::new(),
            reg_keys_local_written_time: Instant::now(),
            reg_key_last_time: Instant::now(),
        }
    }

    /// Debounce the registered keys
    pub async fn debounce(&mut self) {
        let instant = Instant::now();

        for key in self.reg_keys_local_new.iter_mut() {
            if instant >= key.time + KEY_DEBOUNCE {
                #[cfg(feature = "debug")]
                info!("[debounce] debounced key: {}", key.code as u8);
                key.state = KeyState::Released;
            }
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
                        };

                        let registered_key_time = Instant::now();

                        // store the registered key in an the vec
                        if let Some(contained_key) = self
                            .reg_keys_local_new
                            .iter_mut()
                            .find(|k| k.position == new_key_position)
                        {
                            contained_key.time = Instant::now();
                            contained_key.state = KeyState::Pressed;
                        }
                        // else add it
                        else {
                            let _ = self.reg_keys_local_new.push(Key {
                                code: KC::EU,
                                position: KeyPos {
                                    row: row_count as u8,
                                    col: col_count as u8,
                                },
                                time: registered_key_time,
                                state: KeyState::Pressed,
                            });
                        }

                        // store the registered key time
                        self.reg_key_last_time = registered_key_time;
                    }
                }

                // set row to low
                row.set_low();
            }

            // run debounce every 1 ms
            if Self::is_elapsed(&self.reg_keys_local_written_time, Duration::from_millis(1)) {
                // run debouncer
                self.debounce().await;

                if self.reg_keys_local_new != self.reg_keys_local_old {
                    #[cfg(feature = "debug")]
                    info!("[matrix scan] REGISTERED_KEYS sent");
                    REGISTERED_KEYS
                        .sender()
                        .send(self.reg_keys_local_new.iter().cloned().collect())
                        .await;
                }

                if let Some(position) = self
                    .reg_keys_local_new
                    .iter_mut()
                    .position(|k| k.state == KeyState::Released)
                {
                    self.reg_keys_local_new.remove(position);
                }
                self.reg_keys_local_old = self.reg_keys_local_new.iter().cloned().collect();
                self.reg_keys_local_written_time = Instant::now();
            }
        }
    }
}
