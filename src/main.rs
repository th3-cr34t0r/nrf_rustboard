#![no_std]
#![no_main]
mod ble;

use ble::BleControllerBuilder;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{Peri, gpio::Output, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // peripherals init
    let p = embassy_nrf::init(Default::default());

    // ble init
    let ble_builder = BleControllerBuilder::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH19, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23,
        p.PPI_CH24, p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29, p.PPI_CH30,
        p.PPI_CH31, p.RNG, p.RTC0, p.TIMER0, p.TEMP,
    );
    let (sdc, mpsl) = unwrap!(ble_builder.init());
    spawner.must_spawn(run_leds(p.P0_15));
    // todo:
    // spawner.must_spawn(ble_task(sdc, mpsl));
    // spawner.must_spawn(scan_matrix_task());
    // spawner.must_spawn(debounce_task());
}

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
