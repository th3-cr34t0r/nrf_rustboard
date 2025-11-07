#![no_std]
#![no_main]

use nrf_rustboard::{ble::ble_init, matrix::KeyReportLocal};

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_futures::join::join3;

use nrf_rustboard::ble::ble_run;
use nrf_rustboard::peripherals::AppPeri;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let mut p = AppPeri::new();

    // init ble
    let (sdc, mpsl, storage, mut rng) = unwrap!(ble_init(p.ble_peri));

    let mut key_report_local = KeyReportLocal::init();

    // run tasks
    let _ = join3(
        ble_run(sdc, &mpsl, storage, &mut rng, spawner),
        p.matrix_peri.scan(),
        key_report_local.key_provision(),
    )
    .await;
}
