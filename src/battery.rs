#[cfg(feature = "defmt")]
use defmt::info;
use embassy_nrf::{
    Peri,
    peripherals::{P0_04, SAADC},
    saadc::{ChannelConfig, Config, Saadc},
};

use crate::{BATTERY_LEVEL, ble::Irqs, delay_ms};

static BAT_V_TO_PER_TABLE: [(u32, u8); 9] = [
    (3500, 10),
    (3570, 20),
    (3640, 30),
    (3710, 40),
    (3780, 50),
    (3850, 60),
    (3920, 70),
    (3990, 80),
    (4160, 90),
];

pub struct Battery {
    b_percent: u8,
    milli_volts: u32,
    saadc: Saadc<'static, 1>,
}

impl Battery {
    pub fn new(mut p_04: Peri<'static, P0_04>, p_saadc: Peri<'static, SAADC>) -> Self {
        let config = Config::default();
        let channel_configs = ChannelConfig::single_ended(p_04.reborrow());

        let saadc = Saadc::new(p_saadc, Irqs, config, [channel_configs]);
        Self {
            b_percent: 0,
            milli_volts: 0,
            saadc,
        }
    }

    async fn volts_to_percent(&mut self) {
        #[cfg(feature = "defmt")]
        // do the calculation and send over BLE
        info!("[battery_level] voltage: {}", self.milli_volts);

        if self.milli_volts < 3500 {
            self.b_percent = 0;
        } else if self.milli_volts > 4160 {
            self.b_percent = 100;
        } else {
            for (table_milli_v, table_percent) in BAT_V_TO_PER_TABLE.iter() {
                if self.milli_volts < *table_milli_v {
                    self.b_percent = *table_percent;
                    break;
                }
            }
        }
        #[cfg(feature = "defmt")]
        info!("[battery_level] battery: {}%", self.b_percent);
    }

    pub async fn approximate(&mut self) {
        self.saadc.calibrate().await;
        let mut buf = [0; 1];

        let battery_percent_sender = BATTERY_LEVEL.sender();

        delay_ms(1000).await;

        loop {
            let mut avg_buf = 0;

            for i in 1..=10 {
                self.saadc.sample(&mut buf).await;
                avg_buf += buf[0] / i;

                delay_ms(1000).await;
            }

            #[cfg(feature = "defmt")]
            info!("[battery_level] avg_sample: {}", buf[0]);

            self.milli_volts = buf[0] as u32 * (68 * 600) / 4092;
            self.volts_to_percent().await;

            battery_percent_sender.send(self.b_percent);

            // send battery level every 10mins
            delay_ms(600000).await;
        }
    }
}
