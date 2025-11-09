use embassy_time::Duration;

/// Rows per half
pub const ROWS: usize = 4;
/// Cols per half
pub const COLS: usize = 5;
pub const LAYERS: usize = 2;

/// Name your keyboard
pub const BLE_NAME: &str = "Rustboard";

/// Specify if the keyboard is split
pub const SPLIT_PERIPHERAL: bool = true;

pub const ASYNC_ROW_WAIT: u64 = 1; // ms to wait for keypress

pub const REGISTERED_KEYS_BUFFER: usize = 16;

pub const KEY_DEBOUNCE: Duration = Duration::from_millis(20);
