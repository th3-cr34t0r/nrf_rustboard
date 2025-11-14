use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

use crate::{
    KEY_REPORT, LAYER, MATRIX_KEYS, MATRIX_KEYS_BUFFER,
    config::{COLS, LAYERS, ROWS},
    delay_ms,
    keycodes::{KC, KeyType},
    keymap::provide_keymap,
    matrix::{Key, KeyState},
};

pub struct KeyProvision {
    layer: u8,
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
    matrix_keys: Vec<Key, MATRIX_KEYS_BUFFER>,
    keyreport_local: KeyboardReport,
    keyreport_local_old: KeyboardReport,
}

impl KeyProvision {
    pub fn init() -> Self {
        Self {
            layer: 0,
            keymap: provide_keymap(),
            matrix_keys: Vec::new(),
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
                *LAYER.lock().await -= 1;
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
    // /// Debounce the registered keys
    // pub async fn debounce(&mut self) {
    //     let instant = Instant::now();

    //     for key in reg_keys_local_new.iter_mut() {
    //         if instant >= key.time + KEY_DEBOUNCE {
    //             #[cfg(feature = "debug")]
    //             info!("[debounce] debounced key: {}", key.code as u8);
    //             key.state = KeyState::Released;
    //         }
    //     }
    // }

    /// Provision the keys in case of modifiers, combos, macros etc.
    pub async fn run(&mut self) {
        let key_report_sender = KEY_REPORT.sender();

        let mut keys_to_remove: Vec<Key, MATRIX_KEYS_BUFFER> = Vec::new();

        loop {
            // if matrix keys has an element, provision it
            let matrix_keys_locked = MATRIX_KEYS.lock().await;
            if !matrix_keys_locked.is_empty() {
                for key in matrix_keys_locked.iter_mut() {
                    match key.state {
                        KeyState::Pressed => {
                            #[cfg(feature = "debug")]
                            info!(
                                "[key_provision] key.position: col[{}] row[{}] layer[{}]",
                                key.position.col, key.position.row, key.position.layer
                            );

                            // get the keycode
                            key.code = self.keymap[self.layer as usize][key.position.row as usize]
                                [key.position.col as usize];

                            self.provision_pressed_keys(&key.code).await;
                        }
                        KeyState::Released => {
                            #[cfg(feature = "debug")]
                            info!("[key_provision] key released: {}", key.code as u8);

                            // get the keycode
                            key.code = self.keymap[self.layer as usize][key.position.row as usize]
                                [key.position.col as usize];

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
                    if let Some(position) = matrix_keys_locked
                        .iter()
                        .position(|k| k.position == key.position)
                    {
                        matrix_keys_locked.remove(position);
                    }
                }

                // send the report only if different from the old one
                if self.keyreport_local != self.keyreport_local_old {
                    // send report
                    key_report_sender.send(self.keyreport_local).await;

                    self.keyreport_local_old = self.keyreport_local;
                }

                // debounce
                let instant = Instant::now();

                for key in matrix_keys_locked.iter_mut() {
                    if instant >= key.time + KEY_DEBOUNCE {
                        #[cfg(feature = "debug")]
                        info!("[debounce] debounced key: {}", key.code as u8);
                        key.state = KeyState::Released;
                    }
                }
            }
        }

        delay_ms(1).await;
    }
}
