use embassy_time::Duration;

pub const ROWS: usize = 4;
pub const COLS: usize = 10;
pub const LAYERS: usize = 3;

pub const ASYNC_ROW_WAIT: u64 = 1000; // ms to wait for keypress

pub const REGISTERED_KEYS_BUFFER: usize = 16;

pub const KEY_DEBOUNCE: Duration = Duration::from_millis(20);
