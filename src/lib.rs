#![no_std]
#![no_main]

pub mod ble;
pub mod config;
pub mod key_provision;
pub mod keycodes;
pub mod keymap;
pub mod matrix;
pub mod peripherals;
pub mod storage;

use crate::{config::MATRIX_KEYS_BUFFER, matrix::KeyPos};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex},
    mutex::Mutex,
    signal::Signal,
    watch::Watch,
};
use trouble_host::prelude::{CCCD, CccdTable};
use usbd_hid::descriptor::KeyboardReport;

/// Shared variable between ble and key provision tasks
pub static KEY_REPORT: Watch<CriticalSectionRawMutex, KeyboardReport, 2> = Watch::new();

/// Shared variable between ble and key provision tasks
pub static MESSAGE_TO_PERI: Watch<CriticalSectionRawMutex, [u8; 6], 2> = Watch::new();

/// Shared variable between matrix scan and key provision tasks
pub static MATRIX_KEYS: Watch<CriticalSectionRawMutex, [KeyPos; MATRIX_KEYS_BUFFER], 3> =
    Watch::new();

pub static CCCD_TABLE: Signal<CriticalSectionRawMutex, CccdTable<8>> = Signal::new();

use embassy_time::{Duration, Timer};

pub async fn delay_ms(delay: u64) {
    let duration = Duration::from_millis(delay);
    Timer::after(duration).await;
}

pub async fn delay_us(delay: u64) {
    let duration = Duration::from_micros(delay);
    Timer::after(duration).await;
}
