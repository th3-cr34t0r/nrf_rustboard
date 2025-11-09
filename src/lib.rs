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

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use heapless::Vec;
use usbd_hid::descriptor::KeyboardReport;

pub static KEY_REPORT: Channel<ThreadModeRawMutex, KeyboardReport, 2> = Channel::new();
pub static REGISTERED_KEYS: Channel<ThreadModeRawMutex, Vec<Key, { REGISTERED_KEYS_BUFFER }>, 4> =
    Channel::new();

use embassy_time::{Duration, Timer};

use crate::{config::REGISTERED_KEYS_BUFFER, matrix::Key};
pub async fn delay_ms(delay: u64) {
    let duration = Duration::from_millis(delay);
    Timer::after(duration).await;
}

pub async fn delay_us(delay: u64) {
    let duration = Duration::from_micros(delay);
    Timer::after(duration).await;
}
