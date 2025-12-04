use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_time::Instant;
use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

use crate::{
    KEY_REPORT, MATRIX_KEYS_LOCAL, MATRIX_KEYS_SPLIT, MESSAGE_TO_PERI,
    config::{COLS, KEY_DEBOUNCE, LAYERS, MATRIX_KEYS_BUFFER, MATRIX_KEYS_COMB_BUFFER, ROWS},
    delay_ms,
    keycodes::{KC, KeyType},
    keymap::provide_keymap,
    matrix::{Key, KeyPos, KeyState},
};

pub struct KeyProvision {
    #[cfg(feature = "peripheral")]
    layer: u8,
    #[cfg(feature = "peripheral")]
    keymap: [[[KC; COLS * 2]; ROWS]; LAYERS],
    #[cfg(feature = "peripheral")]
    keyreport_local: KeyboardReport,
    #[cfg(feature = "central")]
    message_to_peri_local: [u8; 6],
    #[cfg(feature = "central")]
    message_to_peri_local_old: [u8; 6],
}

#[cfg(feature = "debug")]
use defmt::info;

impl KeyProvision {
    pub fn init() -> Self {
        Self {
            #[cfg(feature = "peripheral")]
            layer: 0,
            #[cfg(feature = "peripheral")]
            keymap: provide_keymap(),
            #[cfg(feature = "peripheral")]
            keyreport_local: KeyboardReport::default(),

            #[cfg(feature = "central")]
            message_to_peri_local: [255; 6],
            #[cfg(feature = "central")]
            message_to_peri_local_old: [255; 6],
        }
    }
    #[cfg(feature = "peripheral")]
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

    #[cfg(feature = "peripheral")]
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
    // async fn debounce(&self, matrix_keys_local: &mut Vec<Key, MATRIX_KEYS_BUFFER>) {
    //     let instant = Instant::now();

    //     for key in matrix_keys_local.iter_mut() {
    //         if instant >= key.time + KEY_DEBOUNCE {
    //             #[cfg(feature = "debug")]
    //             info!("[debounce] debounced key: {}", key.code as u8);
    //             key.state = KeyState::Released;
    //         }
    //     }
    // }

    async fn matrix_to_hid_local(
        &self,
        matrix_keys_local: &mut [Key; MATRIX_KEYS_COMB_BUFFER],
        matrix_keys_received: &[KeyPos; MATRIX_KEYS_BUFFER],
    ) {
        for (index_received, key_pos_received) in matrix_keys_received.iter().enumerate() {
            if *key_pos_received != KeyPos::default() {
                #[cfg(feature = "debug")]
                info!(
                    "[matrix_to_hid] matrix_keys_received: r{} c{}",
                    key_pos.row, key_pos.col
                );

                // if new key is not contained, add it
                if None
                    == matrix_keys_local
                        .iter_mut()
                        .find(|key| key.position == *key_pos_received)
                {
                    let key = Key {
                        #[cfg(feature = "peripheral")]
                        code: self.keymap[self.layer as usize][key_pos_received.row as usize]
                            [key_pos_received.col as usize],

                        #[cfg(feature = "central")]
                        code: KC::EU,
                        position: *key_pos_received,
                        state: KeyState::Pressed,
                    };

                    // set the new key in an empty slot
                    matrix_keys_local[index_received] = key;
                }
            } else {
                if matrix_keys_local[index_received].position != KeyPos::default() {
                    matrix_keys_local[index_received].state = KeyState::Released;
                }
            }
        }
    }

    #[cfg(feature = "peripheral")]
    async fn matrix_to_hid_split(
        &self,
        matrix_keys_local: &mut [Key; MATRIX_KEYS_COMB_BUFFER],
        matrix_keys_received: &[KeyPos; MATRIX_KEYS_BUFFER],
    ) {
        for (index_received, key_pos_received) in matrix_keys_received.iter().enumerate() {
            let index_received = index_received + MATRIX_KEYS_BUFFER;
            if *key_pos_received != KeyPos::default() {
                #[cfg(feature = "debug")]
                info!(
                    "[matrix_to_hid] matrix_keys_received: r{} c{}",
                    key_pos.row, key_pos.col
                );

                // if new key is not contained, add it
                if None
                    == matrix_keys_local
                        .iter_mut()
                        .find(|key| key.position == *key_pos_received)
                {
                    let key = Key {
                        code: self.keymap[self.layer as usize][key_pos_received.row as usize]
                            [key_pos_received.col as usize],
                        position: *key_pos_received,
                        state: KeyState::Pressed,
                    };

                    // set the new key in an empty slot
                    matrix_keys_local[index_received] = key;
                }
            } else {
                if matrix_keys_local[index_received].position != KeyPos::default() {
                    matrix_keys_local[index_received].state = KeyState::Released;
                }
            }
        }
    }
    /// Provision the keys in case of modifiers, combos, macros etc.
    pub async fn run(&mut self) {
        let mut matrix_keys_local_receiver = MATRIX_KEYS_LOCAL
            .receiver()
            .expect("[key_provision] unable to create matrix_key_local_receiver");
        #[cfg(feature = "peripheral")]
        let mut matrix_keys_split_receiver = MATRIX_KEYS_SPLIT
            .receiver()
            .expect("[key_provision] unable to create matrix_key_split_receiver");

        #[cfg(feature = "peripheral")]
        let key_report_sender = KEY_REPORT.sender();
        #[cfg(feature = "central")]
        let message_to_peri = MESSAGE_TO_PERI.sender();

        let mut matrix_keys_local = [Key::default(); MATRIX_KEYS_COMB_BUFFER];
        // let mut matrix_keys_local: Vec<Key, { MATRIX_KEYS_COMB_BUFFER }> = Vec::new();
        let mut keys_to_remove: Vec<Key, MATRIX_KEYS_COMB_BUFFER> = Vec::new();

        loop {
            #[cfg(feature = "peripheral")]
            match select(
                matrix_keys_local_receiver.changed(),
                matrix_keys_split_receiver.changed(),
            )
            .await
            {
                Either::First(matrix_keys_local_received) => {
                    // transform the received local matrix keys
                    self.matrix_to_hid_local(&mut matrix_keys_local, &matrix_keys_local_received)
                        .await;
                }
                Either::Second(matrix_keys_split_received) => {
                    // transform the received split matrix keys
                    self.matrix_to_hid_split(&mut matrix_keys_local, &matrix_keys_split_received)
                        .await;
                }
            }

            #[cfg(feature = "central")]
            {
                let matrix_keys_local_received = matrix_keys_local_receiver.changed().await;
                self.matrix_to_hid_local(&mut matrix_keys_local, &matrix_keys_local_received)
                    .await;
            }

            // process the non default keys to keyreport
            #[cfg(feature = "debug")]
            info!(
                "[key_provision] matrix_keys_local: {:#?}",
                matrix_keys_local
            );
            for key in matrix_keys_local
                .iter_mut()
                .filter(|key| key.position != KeyPos::default())
            {
                match key.state {
                    KeyState::Pressed => {
                        #[cfg(feature = "peripheral")]
                        // get the keycode
                        self.provision_pressed_keys(&key.code).await;

                        #[cfg(feature = "central")]
                        {
                            // set the row and shift 4 bits to left
                            // append the col
                            // row and col must be lower than 16 (fit in 4 bits)
                            let key_to_send = (key.position.row << 4) | key.position.col;

                            if !self.message_to_peri_local.contains(&key_to_send) {
                                if let Some(index) = self
                                    .message_to_peri_local
                                    .iter_mut()
                                    .position(|key| *key == 255)
                                {
                                    self.message_to_peri_local[index] = key_to_send;
                                }
                            }
                        }
                    }
                    KeyState::Released => {
                        #[cfg(feature = "peripheral")]
                        // remove the kc from keyreport_local
                        self.provision_released_keys(&key.code).await;

                        #[cfg(feature = "central")]
                        {
                            // set the row and shift 4 bits to left
                            // append the col
                            // row and col must be lower than 16 (fit in 4 bits)
                            let key_to_rm = (key.position.row << 4) | key.position.col;

                            if let Some(index) = self
                                .message_to_peri_local
                                .iter_mut()
                                .position(|key| *key == key_to_rm)
                            {
                                self.message_to_peri_local[index] = 255;
                            }
                        }

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
                    matrix_keys_local[position] = Key::default();
                }
            }

            // send report
            #[cfg(feature = "peripheral")]
            key_report_sender.send(self.keyreport_local);

            #[cfg(feature = "debug")]
            info!(
                "[key_provision] keyreport_local.keycodes: {:?}",
                self.keyreport_local.keycodes
            );

            #[cfg(feature = "central")]
            {
                if self.message_to_peri_local != self.message_to_peri_local_old {
                    message_to_peri.send(self.message_to_peri_local);
                    #[cfg(feature = "debug")]
                    info!(
                        "[key_provision] message_to_peri_local: {:?}",
                        self.message_to_peri_local
                    );
                    self.message_to_peri_local_old = self.message_to_peri_local;
                }
            }
        }
    }
}
