use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BleConfig {
    pub name: String,
    pub split: bool,
}

#[derive(Deserialize, Debug)]
pub struct MatrixConfig {
    pub rows: usize,
    pub cols: usize,
    // pub row_pins: []
    // pub col_pins: []
}

#[derive(Deserialize, Debug)]
pub struct DebounceConfig {
    pub key_debounce: u64,
}

#[derive(Deserialize, Debug)]
pub struct KeymapConfig {
    pub layers: usize,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub ble: BleConfig,
    pub matrix: MatrixConfig,
    pub debounce: DebounceConfig,
    pub keymap: KeymapConfig,
}
