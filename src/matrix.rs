use crate::config::{ASYNC_ROW_WAIT, COLS, KEY_DEBOUNCE, LAYERS, REGISTERED_KEYS_BUFFER, ROWS};
use crate::keycodes::{KC, KeyType};
use crate::keymap::provide_keymap;
use crate::{KEY_REPORT, LAYER, REGISTERED_KEYS, delay_ms, delay_us};

use core::pin::pin;
use embassy_futures::join::join3;
use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use embassy_time::{Duration, Instant};
use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

#[cfg(feature = "debug")]
use defmt::info;

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

pub struct KeyReportLocal {
    keyreport_local: KeyboardReport,
    keyreport_local_old: KeyboardReport,
}

impl KeyReportLocal {
    pub fn init() -> Self {
        Self {
            keyreport_local: KeyboardReport::default(),
            keyreport_local_old: KeyboardReport::default(),
        }
    }
    pub async fn provision_pressed_keys(&mut self, kc: &KC) {
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
                *LAYER.lock().await = kc.get_layer();
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

    async fn provision_released_keys(&mut self, kc: &KC) {
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
                *LAYER.lock().await -= kc.get_layer();
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
    pub async fn key_provision(&mut self) {
        let key_report_sender = KEY_REPORT.sender();
        let registered_keys_receiver = REGISTERED_KEYS.receiver();
        let registered_keys_sender = REGISTERED_KEYS.sender();

        let mut keys_to_remove: Vec<Key, REGISTERED_KEYS_BUFFER> = Vec::new();

        loop {
            let mut registered_keys_new = registered_keys_receiver.receive().await;

            for key in registered_keys_new.iter_mut() {
                match key.state {
                    KeyState::Pressed => {
                        #[cfg(feature = "debug")]
                        info!(
                            "[key_provision] key.position: col[{}] row[{}] layer[{}]",
                            key.position.col, key.position.row, key.position.layer
                        );
                        self.provision_pressed_keys(&key.code).await;
                    }
                    KeyState::Released => {
                        #[cfg(feature = "debug")]
                        info!("[key_provision] key released: {}", key.code as u8);

                        // remove the kc from keyreport_local
                        self.provision_released_keys(&key.code).await;

                        // remember the key to be removed
                        keys_to_remove
                            .push(*key)
                            .expect("[matrix] keys_to_remove is full");
                    }
                }
            }

            while let Some(key) = keys_to_remove.pop() {
                #[cfg(feature = "debug")]
                info!("[key_provision] keys_to_remove key: {}", key.code as u8);
                if let Some(position) = registered_keys_new
                    .iter()
                    .position(|k| k.position == key.position)
                {
                    registered_keys_new.remove(position);
                }
            }

            // send the report only if different from the old one
            if self.keyreport_local != self.keyreport_local_old {
                registered_keys_sender.send(registered_keys_new).await;
                key_report_sender.send(self.keyreport_local).await;

                self.keyreport_local_old = self.keyreport_local;
            }
        }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    layer: u8,
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
    reg_keys_local_new: Vec<Key, REGISTERED_KEYS_BUFFER>,
    reg_keys_local_old: Vec<Key, REGISTERED_KEYS_BUFFER>,
    keys_sent_time: Instant,
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        Self {
            rows,
            cols,
            layer: 0,
            keymap: provide_keymap(),
            reg_keys_local_new: Vec::new(),
            reg_keys_local_old: Vec::new(),
            keys_sent_time: Instant::now(),
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

    /// Main function for scanning and registering keys
    pub async fn scan(&mut self) {
        loop {
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

            // run debounce every 1 ms
            if Instant::now() >= self.keys_sent_time + Duration::from_millis(1) {
                self.debounce().await;

                if self.reg_keys_local_new != self.reg_keys_local_old {
                    #[cfg(feature = "debug")]
                    info!("[matrix scan] REGISTERED_KEYS sent");
                    REGISTERED_KEYS
                        .sender()
                        .send(self.reg_keys_local_new.iter().cloned().collect())
                        .await;

                    if let Some(position) = self
                        .reg_keys_local_new
                        .iter_mut()
                        .position(|k| k.state == KeyState::Released)
                    {
                        self.reg_keys_local_new.remove(position);
                    }

                    self.reg_keys_local_old = self.reg_keys_local_new.iter().cloned().collect();
                }

                self.keys_sent_time = Instant::now();
            }
        }
    }
}
