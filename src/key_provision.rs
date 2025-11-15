use embassy_futures::select::{Either, select};
use embassy_time::Instant;
use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

use crate::{
    KEY_REPORT, MATRIX_KEYS, MATRIX_KEYS_BUFFER,
    config::{COLS, KEY_DEBOUNCE, LAYERS, ROWS},
    delay_ms,
    keycodes::{KC, KeyType},
    keymap::provide_keymap,
    matrix::{Key, KeyPos, KeyState},
};

pub struct KeyProvision {
    layer: u8,
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
    keyreport_local: KeyboardReport,
    keyreport_local_old: KeyboardReport,
}

#[cfg(feature = "debug")]
use defmt::info;

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

    /// Debounce the registered keys
    async fn debounce(&self, matrix_keys_local: &mut Vec<Key, MATRIX_KEYS_BUFFER>) {
        let instant = Instant::now();

        for key in matrix_keys_local.iter_mut() {
            if instant >= key.time + KEY_DEBOUNCE {
                #[cfg(feature = "debug")]
                info!("[debounce] debounced key: {}", key.code as u8);
                key.state = KeyState::Released;
            }
        }
    }

    async fn matrix_to_hid(
        &self,
        matrix_keys_local: &mut Vec<Key, MATRIX_KEYS_BUFFER>,
        matrix_keys_received: &[KeyPos; MATRIX_KEYS_BUFFER],
    ) {
        for key_pos in matrix_keys_received.iter() {
            if *key_pos != KeyPos::default() {
                #[cfg(feature = "debug")]
                info!(
                    "[matrix_to_hid] matrix_keys_received: r{} c{}",
                    key_pos.row, key_pos.col
                );

                let time = Instant::now();

                if let Some(contained_key) = matrix_keys_local
                    .iter_mut()
                    .find(|key| key.position == *key_pos)
                {
                    contained_key.time = time;
                    contained_key.state = KeyState::Pressed;
                } else {
                    let key = Key {
                        code: self.keymap[self.layer as usize][key_pos.row as usize]
                            [key_pos.col as usize],
                        position: *key_pos,
                        time: time,
                        state: KeyState::Pressed,
                    };
                    matrix_keys_local.push(key).unwrap();
                }

                #[cfg(feature = "debug")]
                {
                    for key in matrix_keys_local.iter() {
                        info!(
                            "[matrix_to_hid] matrix_keys_local: r{} c{}",
                            key.position.row, key.position.col
                        );
                    }
                }
            }
        }
    }
    /// Provision the keys in case of modifiers, combos, macros etc.
    pub async fn run(&mut self) {
        let mut matrix_keys_receiver = MATRIX_KEYS.receiver().expect("[key_provision]");
        let key_report_sender = KEY_REPORT.sender();

        let mut matrix_keys_local: Vec<Key, MATRIX_KEYS_BUFFER> = Vec::new();
        let mut keys_to_remove: Vec<Key, MATRIX_KEYS_BUFFER> = Vec::new();

        loop {
            match select(matrix_keys_receiver.changed(), delay_ms(5)).await {
                Either::First(matrix_keys_received) => {
                    // transform the received matrix keys
                    self.matrix_to_hid(&mut matrix_keys_local, &matrix_keys_received)
                        .await;
                }
                Either::Second(_) => {} // continue with the provisioning
            }

            // process the keys to keyreport
            for key in matrix_keys_local.iter_mut() {
                match key.state {
                    KeyState::Pressed => {
                        // get the keycode
                        self.provision_pressed_keys(&key.code).await;
                    }
                    KeyState::Released => {
                        // remove the kc from keyreport_local
                        self.provision_released_keys(&key.code).await;

                        // remember the key to be removed
                        keys_to_remove
                            .push(*key)
                            .expect("[matrix] keys_to_remove is full");
                    }
                }
            }

            // remove the released keys
            while let Some(key) = keys_to_remove.pop() {
                #[cfg(feature = "debug")]
                info!("[key_provision] keys_to_remove key: {}", key.code as u8);
                if let Some(position) = matrix_keys_local
                    .iter()
                    .position(|k| k.position == key.position)
                {
                    matrix_keys_local.remove(position);
                }
            }

            // send the report only if different from the old one
            if self.keyreport_local != self.keyreport_local_old {
                // send report
                key_report_sender.send(self.keyreport_local);
                #[cfg(feature = "debug")]
                info!(
                    "[key_provision] keyreport_local.keycodes: {:?}",
                    self.keyreport_local.keycodes
                );

                self.keyreport_local_old = self.keyreport_local;
            }

            // debounce
            self.debounce(&mut matrix_keys_local).await;
        }
    }
}
