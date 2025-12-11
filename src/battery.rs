use defmt::info;
use embassy_nrf::{
    Peri,
    peripherals::{P0_04, SAADC},
    saadc::{ChannelConfig, Config, Saadc},
};

use crate::{BATTERY_PERCENT, ble::Irqs, delay_ms};

static ADC_VALUE_PERCENT_TABLE: [(u32, u8); 10] = [(0, 0); 10];

pub struct Battery {
    value: u8,
    p_04: Peri<'static, P0_04>,
    saadc: Peri<'static, SAADC>,
    adc_value_percent: [(u32, u8); 10],
}

impl Battery {
    pub fn new(p_04: Peri<'static, P0_04>, p_saadc: Peri<'static, SAADC>) -> Self {
        Self {
            value: 0,
            p_04,
            saadc: p_saadc,
            adc_value_percent: ADC_VALUE_PERCENT_TABLE,
        }
    }

    // async fn process_measurement(&self, buf: &[i16; 1]) -> u8 {
    //     // do the calculation and send over BLE
    //     0
    // }

    pub async fn measure(&mut self) {
        // resolution = 12bit,
        // oversample = bypass
        let config = Config::default();

        let channel_configs = ChannelConfig::single_ended(self.p_04.reborrow());

        let mut saadc = Saadc::new(self.saadc.reborrow(), Irqs, config, [channel_configs]);

        saadc.calibrate().await;

        let battery_percent_sender = BATTERY_PERCENT.sender();

        let mut buf = [0; 1];

        loop {
            saadc.sample(&mut buf).await;
            info!("[battery_level] sample: {}", buf[0]);
            // let battery_percent = self.process_measurement(&buf).await;

            battery_percent_sender.send(self.value);

            delay_ms(1000).await;
        }
    }
}
