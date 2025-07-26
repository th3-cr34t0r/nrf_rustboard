#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripheral = embassy_nrf::init(Default::default());

    let mut bled = Output::new(peripheral.P0_15, Level::Low, OutputDrive::Standard);

    bled.set_low();

    Timer::after_millis(5000).await;

    loop {
        bled.toggle();
        Timer::after_millis(1000).await;
    }
}
