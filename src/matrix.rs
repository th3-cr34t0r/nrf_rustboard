use crate::config::{ASYNC_ROW_WAIT, COLS, KEY_DEBOUNCE, LAYERS, REGISTERED_KEYS_BUFFER, ROWS};
use crate::keymap::provide_keymap;
use crate::{KEY_REPORT, delay_ms, delay_us};

use core::pin::pin;
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Sender;
use embassy_time::Instant;
use heapless::Vec;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct KeyPos {
    row: u8,
    col: u8,
    layer: u8,
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

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: KeyboardUsage,
    pub position: KeyPos,
    pub time: Instant,
    pub state: KeyState,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KeyboardUsage::KeyboardErrorUndefined,
            position: KeyPos::default(),
            time: Instant::now(),
            state: KeyState::default(),
        }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    layer: u8,
    registered_keys: Vec<Key, REGISTERED_KEYS_BUFFER>,
    keymap: [[COLS; ROWS]; LAYERS],
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        let keymap = provide_keymap();

        Self {
            rows,
            cols,
            layer: 0,
            registered_keys: Vec::new(),
            keymap,
        }
    }

    /// Debounce the registered keys
    async fn debounce_keys(&mut self) {
        let instant = Instant::now();
        self.registered_keys.iter_mut().for_each(|k| {
            if instant >= k.time + KEY_DEBOUNCE {
                k.state = KeyState::Released;
            }
        });
    }

    /// Store new keycodes in the global keyreport
    async fn local_to_global_keyreport(
        &mut self,
        key_report_sender: Sender<'a, CriticalSectionRawMutex, KeyboardReport, 2>,
    ) {
        let mut keys_to_remove: Vec<KeyboardUsage, REGISTERED_KEYS_BUFFER> = Vec::new();

        for key in self.registered_keys.iter_mut() {
            match key.state {
                KeyState::Pressed => {
                    // in case the key code is not contained in the key_report, add it
                    if let Some(key_report_sender) = key_report_sender.try_get() {
                        if let None = key_report_sender
                            .keycodes
                            .iter()
                            .find(|kc| *kc == &(key.code as u8))
                        {
                            if let Some(position) = key_report
                                .keycodes
                                .iter_mut()
                                .position(|element| element == &(0 as u8))
                            {
                                key_report_sender.send_modify(|key_report| {
                                    key_report.unwrap().keycodes[position] = key.code as u8
                                });
                            }
                        }
                    }
                }
                KeyState::Released => {
                    if let Some(key_report_sender) = key_report_sender.try_get() {
                        if let Some(position) = key_report_sender
                            .keycodes
                            .iter()
                            .position(|kc| *kc == key.code as u8)
                        {
                            key_report_sender.send_modify(|key_report| {
                                key_report.unwrap().keycodes[position] = 0 as u8
                            });

                            // remember the key to be removed
                            keys_to_remove
                                .push(key.code)
                                .expect("[matrix] keys_to_remove is full");
                        }
                    }
                }
            }
        }

        // remove the released keys
        while let Some(kc) = keys_to_remove.pop() {
            if let Some(position) = self.registered_keys.iter().position(|k| k.code == kc) {
                self.registered_keys.remove(position);
            }
        }
    }

    /// Main function for scanning and registering keys
    pub async fn scan(&mut self) {
        for (row_count, row) in self.rows.iter_mut().enumerate() {
            row.set_high();
            // delay so port propagates
            delay_us(1).await;

            // set cols wait for high
            {
                let mut futures: Vec<_, COLS> = self
                    .cols
                    .iter_mut()
                    .map(|col| col.wait_for_high())
                    .collect();

                match select(
                    select_slice(pin!(futures.as_mut_slice())),
                    delay_ms(ASYNC_ROW_WAIT),
                )
                .await
                {
                    Either::First(_) => {
                        // key has been pressed
                    }
                    Either::Second(()) => {
                        // time is up, continue with the next row
                        row.set_low();
                        continue;
                    }
                }
            }

            // get the pressed keys
            for (col_count, col) in self.cols.iter().enumerate() {
                if col.is_high() {
                    let new_key_position = KeyPos {
                        row: row_count as u8,
                        col: col_count as u8,
                        layer: self.layer,
                    };

                    // store the registered key in an the vec
                    if let Some(contained_key) = self
                        .registered_keys
                        .iter_mut()
                        .find(|k| k.position == new_key_position)
                    {
                        // if the key is already contained, update it
                        if (contained_key.position
                            == KeyPos {
                                row: row_count as u8,
                                col: col_count as u8,
                                layer: self.layer,
                            })
                        {
                            contained_key.time = Instant::now();
                            contained_key.state = KeyState::Pressed;
                        }
                    }
                    // else add it
                    else {
                        let _ = self.registered_keys.push(Key {
                            code: self.keymap[col_count][row_count][self.layer],
                            position: KeyPos {
                                row: row_count as u8,
                                col: col_count as u8,
                                layer: self.layer,
                            },
                            time: Instant::now(),
                            state: KeyState::Pressed,
                        });
                    }
                }
            }

            // set row to low
            row.set_low();
        }
    }
}

pub async fn scan_matrix<'a>(mut matrix_peri: Matrix<'a>) {
    let mut key_report_sender = KEY_REPORT.sender();

    loop {
        // run the matrix scan
        matrix_peri.scan().await;

        // debounce
        matrix_peri.debounce_keys().await;

        // store to global keyreport
        matrix_peri
            .local_to_global_keyreport(&mut key_report_sender)
            .await;
    }
}
