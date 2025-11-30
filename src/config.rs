use embassy_time::Duration;

/// Rows per half
pub const ROWS: usize = 4;
/// Cols per half
pub const COLS: usize = 5;
pub const LAYERS: usize = 2;

/// Name your keyboard
pub const BLE_NAME: &str = "Rustboard";

pub const PERI_ADDRESS: [u8; 6] = [0x0c, 0x4d, 0x2e, 0xb4, 0x1d, 0xfb];

/// Specify if the keyboard is split
pub const SPLIT_PERIPHERAL: bool = true;

pub const MATRIX_KEYS_BUFFER: usize = 6;

/// Wait for a given time before entering sleep in ms
pub const ENTER_SLEEP_DEBOUNCE: u64 = 600000;

/// Enter interupt on key debounce
pub const KEY_INTERUPT_DEBOUNCE: Duration = Duration::from_millis(1);

/// Debounce key in ms
pub const KEY_DEBOUNCE: Duration = Duration::from_millis(20);
