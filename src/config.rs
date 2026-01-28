/// Peripheral address for connecting central to peripheral
pub const PERI_ADDRESS: [u8; 6] = [0x0c, 0x4d, 0x2e, 0xb4, 0x1d, 0xfb];

/// Size of the registered matrix keys array
pub const MATRIX_KEYS_BUFFER: usize = 6;

/// Size of the registered matrix keys array for both halfs
pub const MATRIX_KEYS_COMB_BUFFER: usize = MATRIX_KEYS_BUFFER * 2;

/// Wait for a given time before entering sleep in ms
pub const ENTER_SLEEP_DEBOUNCE: u64 = 600000;
