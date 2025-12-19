use embassy_time::Duration;

/// Rows per half
pub const ROWS: usize = 4;
/// Cols per half
pub const COLS: usize = 5;
/// KeyMap total Layers
pub const LAYERS: usize = 2;

/// Name your keyboard
pub const BLE_NAME: &str = "Rustboard";

/// Peripheral address for connecting central to peripheral
pub const PERI_ADDRESS: [u8; 6] = [0x0c, 0x4d, 0x2e, 0xb4, 0x1d, 0xfb];

/// Specify if the keyboard is split
pub const SPLIT_PERIPHERAL: bool = true;

/// Size of the registered matrix keys array
pub const MATRIX_KEYS_BUFFER: usize = 6;

/// Size of the registered matrix keys array for both halfs
pub const MATRIX_KEYS_COMB_BUFFER: usize = MATRIX_KEYS_BUFFER * 2;

/// Wait for a given time before entering sleep in ms
pub const ENTER_SLEEP_DEBOUNCE: u64 = 600000;

/// Key debounce period
pub const KEY_DEBOUNCE: Duration = Duration::from_millis(10);
