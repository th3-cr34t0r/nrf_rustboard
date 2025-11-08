use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

use crate::{
    KEY_REPORT, REGISTERED_KEYS, REGISTERED_KEYS_BUFFER,
    config::{COLS, LAYERS, ROWS},
    keycodes::{KC, KeyType},
    keymap::provide_keymap,
    matrix::{Key, KeyState},
};

pub struct KeyProvision {
    layer: u8,
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
    keyreport_local: KeyboardReport,
    keyreport_local_old: KeyboardReport,
}

impl KeyProvision {
    pub fn init() -> Self {
        Self {
            layer: 0,
            keymap: provide_keymap(),
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
    pub async fn run(&mut self) {
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
