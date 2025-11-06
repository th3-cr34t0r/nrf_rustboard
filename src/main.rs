#![no_std]
#![no_main]

use nrf_rustboard::ble::ble_init;
use nrf_rustboard::matrix::scan_matrix;

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_futures::join::join;

use nrf_rustboard::ble::ble_run;
use nrf_rustboard::peripherals::AppPeri;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let p = AppPeri::new();

    // init ble
    let (sdc, mpsl, storage, mut rng) = unwrap!(ble_init(p.ble_peri));

    // run tasks
    let _ = join(
        ble_run(sdc, &mpsl, storage, &mut rng, spawner),
        scan_matrix(p.matrix_peri),
    )
    .await;
}
