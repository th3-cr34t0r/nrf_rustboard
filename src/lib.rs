#![no_std]
#![no_main]

pub mod battery;
pub mod ble;
pub mod config;
pub mod key_provision;
pub mod keycodes;
pub mod keymap;
pub mod matrix;
pub mod peripherals;
pub mod storage;

use crate::{config::MATRIX_KEYS_BUFFER, matrix::KeyPos};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

/// Shared variable between matrix scan and key provision tasks
pub static MATRIX_KEYS_LOCAL: Watch<CriticalSectionRawMutex, [KeyPos; MATRIX_KEYS_BUFFER], 2> =
    Watch::new();

#[cfg(feature = "peripheral")]
use usbd_hid::descriptor::KeyboardReport;

#[cfg(feature = "peripheral")]
/// Shared variable between ble and key provision tasks
pub static KEY_REPORT: Watch<CriticalSectionRawMutex, KeyboardReport, 2> = Watch::new();

#[cfg(feature = "peripheral")]
/// Shared variable between matrix scan and key provision tasks
pub static MATRIX_KEYS_SPLIT: Watch<CriticalSectionRawMutex, [KeyPos; MATRIX_KEYS_BUFFER], 2> =
    Watch::new();

#[cfg(feature = "central")]
/// Shared variable between ble and key provision tasks
pub static MESSAGE_TO_PERI: Watch<CriticalSectionRawMutex, [u8; 6], 2> = Watch::new();

/// Shared variable for battery percentage information
pub static BATTERY_LEVEL: Watch<CriticalSectionRawMutex, u8, 3> = Watch::new();

use embassy_time::{Duration, Timer};

pub async fn delay_ms(delay: u64) {
    let duration = Duration::from_millis(delay);
    Timer::after(duration).await;
}

pub async fn delay_us(delay: u64) {
    let duration = Duration::from_micros(delay);
    Timer::after(duration).await;
}
