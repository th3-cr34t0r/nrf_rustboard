#![no_std]
#![no_main]

mod ble;
mod config;
mod debounce;
mod key_provision;
mod matrix;
mod peripherals;

use ble::ble_init;
use debounce::debounce;
use key_provision::key_provision;
use matrix::scan_matrix;

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_futures::join::join4;

use crate::ble::ble_run;
use peripherals::AppPeri;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let p = AppPeri::new();

    // init ble
    let (sdc, mpsl, storage, mut rng) = unwrap!(ble_init(p.ble_peri));

    // run tasks
    let _ = join4(
        ble_run(sdc, &mpsl, storage, &mut rng, spawner),
        scan_matrix(p.rows, p.cols),
        debounce(),
        key_provision(),
    )
    .await;
}
