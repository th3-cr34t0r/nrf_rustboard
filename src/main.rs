#![no_std]
#![no_main]
mod ble;

use ble::BleControllerBuilder;
use defmt::unwrap;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let ble_builder = BleControllerBuilder::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH19, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23,
        p.PPI_CH24, p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29, p.PPI_CH30,
        p.PPI_CH31, p.RNG, p.RTC0, p.TIMER0, p.TEMP,
    );

    let (sdc, mpsl) = unwrap!(ble_builder.init());
}
