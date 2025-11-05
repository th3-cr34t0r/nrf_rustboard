#![no_std]
#![no_main]

pub mod ble;
pub mod config;
pub mod key_provision;
pub mod keycodes;
pub mod keymap;
pub mod matrix;
pub mod peripherals;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use usbd_hid::descriptor::KeyboardReport;

pub static KEY_REPORT: Mutex<CriticalSectionRawMutex, KeyboardReport> =
    Mutex::new(KeyboardReport::default());

use embassy_time::{Duration, Timer};
pub async fn delay_ms(delay: u64) {
    let duration = Duration::from_millis(delay);
    Timer::after(duration).await;
}

pub async fn delay_us(delay: u64) {
    let duration = Duration::from_micros(delay);
    Timer::after(duration).await;
}
