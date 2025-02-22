#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripheral = embassy_nrf::init(Default::default());

    let mut led = Output::new(peripheral.P0_13, Level::Low, OutputDrive::Standard);

    loop {
        led.toggle();
        Timer::after_millis(300).await;
    }
}
