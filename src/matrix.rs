use crate::config::{COLS, ENTER_SLEEP_DEBOUNCE, KEY_INTERUPT_DEBOUNCE, MATRIX_KEYS_BUFFER, ROWS};
use crate::keycodes::KC;
use crate::{MATRIX_KEYS, delay_ms, delay_us};

use core::pin::pin;
use defmt::{Format, info};
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_time::{Duration, Instant};
use heapless::Vec;

#[cfg(feature = "debug")]
use defmt::info;

#[derive(Debug, Default, Clone, Copy, PartialEq, Format)]
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
    registered_keys: [KeyPos; MATRIX_KEYS_BUFFER],
    reg_keys_local_written_time: Instant,
    reg_key_last_time: Instant,
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            registered_keys: [KeyPos::default(); MATRIX_KEYS_BUFFER],
            reg_keys_local_written_time: Instant::now(),
            reg_key_last_time: Instant::now(),
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
        let matrix_keys_sender = MATRIX_KEYS.sender();

        loop {
            if Self::is_elapsed(&self.reg_key_last_time, KEY_INTERUPT_DEBOUNCE) {
                for row in self.rows.iter_mut() {
                    row.set_high();
                    // delay so port propagates
                    delay_us(1).await;
                }

                // set cols wait for high
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
                        for row in self.rows.iter_mut() {
                            // set rows to low
                            row.set_low();
                        }
                    }
                    Either::Second(()) => {
                        // enter sleep
                        // TODO:
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

                        // add the new key position only if it is not contained
                        if !self.registered_keys.contains(&new_key_position) {
                            // add it to a free slot
                            if let Some(index) = self
                                .registered_keys
                                .iter()
                                .position(|&key_pos| key_pos == KeyPos::default())
                            {
                                self.registered_keys[index] = new_key_position;
                            };

                            self.reg_key_last_time = Instant::now();
                        }
                    }
                }

                // set row to low
                row.set_low();

                // trottle down the scanning to perserve battery
                delay_ms(1).await;
            }

            // send the new value
            if self
                .registered_keys
                .iter()
                .any(|&l_kp| l_kp != KeyPos::default())
            {
                matrix_keys_sender.send(self.registered_keys.clone());

                info!(
                    "[matrix scan] self.reg_keys_local_new: {:?}",
                    self.registered_keys
                );

                // reset the array
                self.registered_keys.fill(KeyPos::default());
            }

            self.reg_keys_local_written_time = Instant::now();
        }
    }
}
