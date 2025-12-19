use crate::config::{COLS, ENTER_SLEEP_DEBOUNCE, KEY_DEBOUNCE, MATRIX_KEYS_BUFFER, ROWS};
use crate::key_provision::KeyProvision;
use crate::keycodes::KC;
use crate::{MATRIX_KEYS_LOCAL, delay_ms, delay_us};

use core::pin::pin;
#[cfg(feature = "defmt")]
use defmt::{Format, info};
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_time::Instant;
use heapless::Vec;

#[cfg_attr(feature = "defmt", derive(Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyPos {
    pub row: u8,
    pub col: u8,
}

impl KeyPos {
    pub fn default() -> Self {
        Self { row: 255, col: 255 }
    }
}

#[cfg_attr(feature = "defmt", derive(Format))]
#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum KeyState {
    #[default]
    Released,
    Pressed,
}

#[cfg_attr(feature = "defmt", derive(Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub code: KC,
    pub position: KeyPos,
    pub state: KeyState,
    pub time: Instant,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KC::default(),
            position: KeyPos::default(),
            state: KeyState::default(),
            time: Instant::now(),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(Format))]
#[derive(Copy, Clone, PartialEq)]
struct MatrixKey {
    keypos: KeyPos,
    time: Instant,
}

impl Default for MatrixKey {
    fn default() -> Self {
        Self {
            keypos: KeyPos::default(),
            time: Instant::now(),
        }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    reg_keys: [MatrixKey; MATRIX_KEYS_BUFFER],
    keys_to_send_new: [KeyPos; MATRIX_KEYS_BUFFER],
    keys_to_send_old: [KeyPos; MATRIX_KEYS_BUFFER],
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            reg_keys: [MatrixKey::default(); MATRIX_KEYS_BUFFER],
            keys_to_send_new: [KeyPos::default(); MATRIX_KEYS_BUFFER],
            keys_to_send_old: [KeyPos::default(); MATRIX_KEYS_BUFFER],
        }
    }

    /// Debounce the registered keys
    async fn debouncer(&mut self) {
        let instant = Instant::now();

        for c_key in self
            .reg_keys
            .iter_mut()
            .filter(|c_key| c_key.keypos != KeyPos::default())
        {
            if instant >= c_key.time + KEY_DEBOUNCE {
                #[cfg(feature = "defmt")]
                info!("[debounce] debounced key: {:?}", c_key.keypos);
                c_key.keypos = KeyPos::default();
            }
        }
    }

    /// Main function for scanning and registering keys
    pub async fn scan(&mut self) {
        let matrix_keys_sender = MATRIX_KEYS_LOCAL.sender();

        loop {
            if self
                .reg_keys
                .iter()
                .all(|m_key| m_key.keypos == KeyPos::default())
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
                delay_us(10).await;

                // get the pressed keys
                for (col_count, col) in self.cols.iter().enumerate() {
                    if col.is_high() {
                        let new_m_key = MatrixKey {
                            keypos: KeyPos {
                                row: row_count as u8,
                                col: col_count as u8,
                            },

                            time: Instant::now(),
                        };

                        // add the new key position only if it is not contained
                        if self
                            .reg_keys
                            .iter()
                            .find(|c_key| c_key.keypos == new_m_key.keypos)
                            .is_none()
                        {
                            // add it to a free slot
                            if let Some(index) = self
                                .reg_keys
                                .iter()
                                .position(|&key_pos| key_pos.keypos == KeyPos::default())
                            {
                                self.reg_keys[index] = new_m_key;
                            };
                        }
                        // update its time
                        else {
                            if let Some(index) = self
                                .reg_keys
                                .iter()
                                .position(|c_key| c_key.keypos == new_m_key.keypos)
                            {
                                self.reg_keys[index].time = Instant::now();
                            }
                        }
                    }
                }

                // set row to low
                row.set_low();

                // we aim at 1ms scan interval per scan
                delay_us(1000 / ROWS as u64).await;
            }

            // debouncer
            self.debouncer().await;

            // filter all non defalut KeyPos elements
            for (index, c_key) in self.reg_keys.iter().enumerate() {
                self.keys_to_send_new[index] = c_key.keypos;
            }

            // send the new value
            if self.keys_to_send_new != self.keys_to_send_old {
                #[cfg(feature = "defmt")]
                info!("[matrix] sent keys: {:?}", self.keys_to_send_new);

                // send the keys
                matrix_keys_sender.send(self.keys_to_send_new);

                self.keys_to_send_old = self.keys_to_send_new;
            }
        }
    }
}
