#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join4;
use nrf_rustboard::battery::Battery;
use nrf_rustboard::{ble::ble_init_run, key_provision::KeyProvision};

use nrf_rustboard::peripherals::AppPeri;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let mut p = AppPeri::new();
    let mut battery = Battery::new(p.p04, p.saadc);

    // init key provision
    let mut key_provision = KeyProvision::init();

    // run tasks
    let _ = join4(
        ble_init_run(p.ble_peri, spawner),
        p.matrix_peri.scan(),
        key_provision.run(),
        battery.process(),
    )
    .await;
}
