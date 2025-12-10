use defmt::info;
use embassy_nrf::{
    Peri,
    peripherals::{P0_04, SAADC},
    saadc::{ChannelConfig, Config, Saadc},
};

use crate::{ble::Irqs, delay_ms};

pub struct Battery {
    value: u8,
    p_04: Peri<'static, P0_04>,
    saadc: Peri<'static, SAADC>,
}

impl Battery {
    pub fn new(p_04: Peri<'static, P0_04>, p_saadc: Peri<'static, SAADC>) -> Self {
        Self {
            value: 0,
            p_04,
            saadc: p_saadc,
        }
    }

    pub async fn process(&mut self) {
        let config = Config::default();
        let channel_configs = ChannelConfig::single_ended(self.p_04.reborrow());
        let mut saadc = Saadc::new(self.saadc.reborrow(), Irqs, config, [channel_configs]);

        saadc.calibrate().await;

        let mut buf = [0; 1];

        loop {
            saadc.sample(&mut buf).await;
            info!("[battery_level] sample: {}", buf[0]);
            delay_ms(600000).await;
        }
    }
}
