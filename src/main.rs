#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join3;
use nrf_rustboard::{ble::ble_init_run, key_provision::KeyProvision, peripherals::AppPeri};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let mut p = AppPeri::new();

    // init key provision
    let mut key_provision = KeyProvision::init();

    // run tasks
    let _ = join3(
        ble_init_run(p.ble_peri, spawner),
        p.matrix_peri.scan(),
        key_provision.run(),
    )
    .await;
}
