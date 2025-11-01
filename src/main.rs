#![no_std]
#![no_main]

mod ble;
mod debounce;
mod key_provision;
mod matrix;

use ble::ble_init;
use debounce::debounce::debounce;
use key_provision::key_provision::key_provision;
use matrix::scan_matrix::scan_matrix;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select::select3;
use embassy_nrf::{Peri, gpio::Output, peripherals};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
pub async fn run_leds(led_pin: Peri<'static, peripherals::P0_15>) -> ! {
    let mut led = Output::new(
        led_pin,
        embassy_nrf::gpio::Level::Low,
        embassy_nrf::gpio::OutputDrive::Standard,
    );

    loop {
        led.toggle();
        info!("Led toggled");
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let p = embassy_nrf::init(Default::default());

    // init ble
    let (sdc, mpsl, storage, mut rng) = unwrap!(ble_init(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH19, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23,
        p.PPI_CH24, p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29, p.PPI_CH30,
        p.PPI_CH31, p.RTC0, p.TIMER0, p.TEMP, p.NVMC, p.RNG
    ));

    // run ble
    ble::run(sdc, &mpsl, storage, &mut rng, spawner).await;

    let _ = select3(scan_matrix(), debounce(), key_provision()).await;
}
