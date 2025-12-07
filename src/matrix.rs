use crate::config::{COLS, ENTER_SLEEP_DEBOUNCE, MATRIX_KEYS_BUFFER, ROWS};
use crate::keycodes::KC;
use crate::{MATRIX_KEYS_LOCAL, delay_ms, delay_us};

use core::pin::pin;
use defmt::Format;
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_time::Instant;
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

#[derive(Default, PartialEq, Debug, Clone, Copy, Format)]
pub enum KeyState {
    #[default]
    Released,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Format)]
pub struct Key {
    pub code: KC,
    pub position: KeyPos,
    pub state: KeyState,
    pub time: Instant,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KC::EU,
            position: KeyPos::default(),
            state: KeyState::default(),
            time: Instant::now(),
        }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    registered_keys_new: [KeyPos; MATRIX_KEYS_BUFFER],
    registered_keys_old: [KeyPos; MATRIX_KEYS_BUFFER],
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            registered_keys_new: [KeyPos::default(); MATRIX_KEYS_BUFFER],
            registered_keys_old: [KeyPos::default(); MATRIX_KEYS_BUFFER],
        }
    }

    /// Main function for scanning and registering keys
    pub async fn scan(&mut self) {
        let matrix_keys_sender = MATRIX_KEYS_LOCAL.sender();

        loop {
            if self
                .registered_keys_new
                .iter()
                .all(|key_pos| *key_pos == KeyPos::default())
            {
                for row in self.rows.iter_mut() {
                    row.set_high();
                    // delay so port propagates
                    delay_us(1).await;
                }

                // set cols wait for high
                let mut futures: Vec<_, COLS> = self
                    .cols
                    .iter_mut()
                    .map(|col| col.wait_for_any_edge())
                    .collect();

                match select(
                    select_slice(pin!(futures.as_mut_slice())),
                    delay_ms(ENTER_SLEEP_DEBOUNCE),
                )
                .await
                {
                    Either::First(_) => {
                        // key has been pressed, but first set all rows to low
                        for row in self.rows.iter_mut() {
                            row.set_low();
                        }
                    }
                    Either::Second(()) => {
                        // enter sleep
                        // TODO:
                    }
                }
            }

            // run matrix scan
            for (row_count, row) in self.rows.iter_mut().enumerate() {
                row.set_high();
                // delay so port propagates
                delay_us(250).await;

                // get the pressed keys
                for (col_count, col) in self.cols.iter().enumerate() {
                    if col.is_high() {
                        let new_key_position = KeyPos {
                            row: row_count as u8,
                            col: col_count as u8,
                        };

                        // add the new key position only if it is not contained
                        if !self.registered_keys_new.contains(&new_key_position) {
                            // add it to a free slot
                            if let Some(index) = self
                                .registered_keys_new
                                .iter()
                                .position(|&key_pos| key_pos == KeyPos::default())
                            {
                                self.registered_keys_new[index] = new_key_position;
                            };
                        }
                    } else {
                        let new_key_position = KeyPos {
                            row: row_count as u8,
                            col: col_count as u8,
                        };
                        if let Some(index) = self
                            .registered_keys_new
                            .iter()
                            .position(|key_pos| *key_pos == new_key_position)
                        {
                            self.registered_keys_new[index] = KeyPos::default();
                        }
                    }
                }

                // set row to low
                row.set_low();
            }

            // send the new value
            if self.registered_keys_new != self.registered_keys_old {
                #[cfg(feature = "debug")]
                info!("[matrix] sent keys: {:?}", self.registered_keys_new);
                matrix_keys_sender.send(self.registered_keys_new.clone());

                self.registered_keys_old = self.registered_keys_new;
            }
        }
    }
}
