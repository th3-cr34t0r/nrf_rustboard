use crate::config::{ASYNC_ROW_WAIT, COLS, KEY_DEBOUNCE, LAYERS, REGISTERED_KEYS_BUFFER, ROWS};
use crate::keycodes::{KC, KeyType};
use crate::keymap::provide_keymap;
use crate::{KEY_REPORT, delay_ms, delay_us};

use core::pin::pin;
use defmt::info;
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Sender;
use embassy_time::Instant;
use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

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
    layer: u8,
    registered_keys: Vec<Key, REGISTERED_KEYS_BUFFER>,
    keyreport_local: KeyboardReport,
    keyreport_local_old: KeyboardReport,
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            layer: 0,
            registered_keys: Vec::new(),
            keyreport_local: KeyboardReport::default(),
            keyreport_local_old: KeyboardReport::default(),
            keymap: provide_keymap(),
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
                        contained_key.time = Instant::now();
                        contained_key.state = KeyState::Pressed;
                    }
                    // else add it
                    else {
                        let _ = self.registered_keys.push(Key {
                            code: self.keymap[self.layer as usize][row_count as usize]
                                [col_count as usize],
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

    /// Debounce the registered keys
    async fn debounce_keys(&mut self) {
        let instant = Instant::now();
        self.registered_keys.iter_mut().for_each(|k| {
            if instant >= k.time + KEY_DEBOUNCE {
                k.state = KeyState::Released;
            }
        });
    }

    fn provision_pressed_keys(&mut self, kc: &KC) {
        // get the key type
        match KeyType::check_type(kc) {
            // KeyType::Combo => {
            //     let (combo_valid_keys, _keys_to_remove) = Kc::get_combo(hid_key);
            //     for valid_key in combo_valid_keys.iter() {
            //         add_keys_master(keyboard_key_report, mouse_key_report, valid_key, layer);
            //     }
            // }
            // KeyType::Macro => {
            //     let macro_valid_keys = Kc::get_macro_sequence(hid_key);
            //     for valid_key in macro_valid_keys.iter() {
            //         add_keys_master(keyboard_key_report, mouse_key_report, valid_key, layer);
            //     }
            // }
            KeyType::Layer => {
                // check and set the layer
                self.layer = kc.get_layer();
            }
            KeyType::Modifier => {
                self.keyreport_local.modifier |= kc.get_modifier();
            }
            // KeyType::Mouse => {
            //     // set the mouse command to the mouse ble characteristic
            //     mouse_key_report.set_command(hid_key);
            // }
            KeyType::Key => {
                // check if the key count is less than 6
                if !self.keyreport_local.keycodes.contains(&(*kc as u8)) {
                    // find the first key slot in the array that is free
                    if let Some(index) = self
                        .keyreport_local
                        .keycodes
                        .iter()
                        .position(|&value| value == 0)
                    {
                        // add the new key to that position
                        self.keyreport_local.keycodes[index] = *kc as u8
                    }
                }
            }

            _ => {} // TODO: temporary
        }
    }

    fn provision_released_keys(&mut self, kc: &KC) {
        // get the key type
        match KeyType::check_type(kc) {
            //     KeyType::Combo => {
            //         let (combo_valid_keys, _keys_to_change) = Kc::get_combo(hid_key);
            //         for valid_key in combo_valid_keys.iter() {
            //             remove_keys_master(keyboard_key_report, mouse_key_report, valid_key, layer);
            //         }
            //     }

            //     KeyType::Macro => {
            //         let macro_valid_keys = Kc::get_macro_sequence(hid_key);
            //         for valid_key in macro_valid_keys.iter() {
            //             remove_keys_master(keyboard_key_report, mouse_key_report, valid_key, layer);
            //         }
            //     }
            KeyType::Layer => {
                // set previous layer
                self.layer -= 1;
            }
            KeyType::Modifier => {
                // remove the modifier
                self.keyreport_local.modifier &= !kc.get_modifier();
            }
            // KeyType::Mouse => {
            //     // remove the mouse command from the mouse ble characteristic
            //     mouse_key_report.reset_keypress(hid_key);
            // }
            KeyType::Key => {
                // find the key index of the released key
                if let Some(index) = self
                    .keyreport_local
                    .keycodes
                    .iter()
                    .position(|&value| value == *kc as u8)
                {
                    // remove the key from the keyreport_local
                    self.keyreport_local.keycodes[index] = 0;
                }
            }
            _ => {}
        }
    }
    /// Provision the keys in case of modifiers, combos, macros etc.
    async fn key_provision(&mut self) {
        let mut keys_to_add: Vec<KC, REGISTERED_KEYS_BUFFER> = Vec::new();

        let mut keys_to_remove: Vec<Key, REGISTERED_KEYS_BUFFER> = Vec::new();

        for key in self.registered_keys.iter_mut() {
            match key.state {
                KeyState::Pressed => {
                    #[cfg(feature = "debug")]
                    info!(
                        "[key provision] key.position: col[{}] row[{}] layer[{}]",
                        key.position.col, key.position.row, key.position.layer
                    );

                    // remember the key to be added
                    keys_to_add
                        .push(key.code)
                        .expect("[matrix] keys_to_add is full");
                }
                KeyState::Released => {
                    // remember the key to be removed
                    keys_to_remove
                        .push(*key)
                        .expect("[matrix] keys_to_remove is full");
                }
            }
        }

        // add the pressed keys
        for key_code in keys_to_add {
            self.provision_pressed_keys(&key_code);
        }

        // remove the released keys
        for key in keys_to_remove {
            // remove the kc from keyreport_local
            self.provision_released_keys(&key.code);

            // remove the key from registered_keys
            if let Some(position) = self
                .registered_keys
                .iter()
                .position(|k| k.position == key.position)
            {
                self.registered_keys.remove(position);
            }
        }
    }

    /// Store local keycodes in globally shared keyreport
    async fn local_to_global(
        &mut self,
        keyreport_sender: &mut Sender<'a, CriticalSectionRawMutex, KeyboardReport, 2>,
    ) {
        // send the report only if different from the old one
        if self.keyreport_local != self.keyreport_local_old {
            keyreport_sender.send(self.keyreport_local);

            self.keyreport_local_old = self.keyreport_local;
        }
    }
}

pub async fn scan_matrix<'a>(mut matrix_peri: Matrix<'a>) {
    let mut key_report_sender = KEY_REPORT.sender();

    loop {
        // run the matrix scan
        matrix_peri.scan().await;

        // store to global keyreport
        matrix_peri.key_provision().await;

        // store the local in global keycodes
        matrix_peri.local_to_global(&mut key_report_sender).await;

        // debounce
        matrix_peri.debounce_keys().await;
    }
}
