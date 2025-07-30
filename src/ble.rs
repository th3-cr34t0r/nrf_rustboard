use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{self, RNG};
use embassy_nrf::{Peri, bind_interrupts, rng};
use nrf_mpsl::Peripherals as mpsl_Peripherals;
use nrf_mpsl::raw::{
    MPSL_CLOCK_LF_SRC_RC, MPSL_DEFAULT_CLOCK_ACCURACY_PPM, MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED,
    MPSL_RECOMMENDED_RC_CTIV, MPSL_RECOMMENDED_RC_TEMP_CTIV,
};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{
    self as sdc, Mem, Peripherals as sdc_Peripherals, SoftdeviceController,
    mpsl::{
        ClockInterruptHandler, HighPrioInterruptHandler, LowPrioInterruptHandler,
        raw::mpsl_clock_lfclk_cfg_t,
    },
};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => LowPrioInterruptHandler;
    CLOCK_POWER => ClockInterruptHandler;
    RADIO => HighPrioInterruptHandler;
    TIMER0 => HighPrioInterruptHandler;
    RTC0 => HighPrioInterruptHandler;
});

/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 1112; // bytes

pub struct BleControllerBuilder<'a> {
    sdc_p: sdc_Peripherals<'a>,
    sdc_mem: Mem<SDC_MEMORY_SIZE>,
    //
    ppi_ch19: Peri<'a, peripherals::PPI_CH19>,
    ppi_ch30: Peri<'a, peripherals::PPI_CH30>,
    ppi_ch31: Peri<'a, peripherals::PPI_CH31>,
    rng_p: Peri<'a, peripherals::RNG>,
    rtc0: Peri<'a, peripherals::RTC0>,
    timer0: Peri<'a, peripherals::TIMER0>,
    temp: Peri<'a, peripherals::TEMP>,
}
impl<'a> BleControllerBuilder<'a>
where
    'a: 'static,
{
    const LFCLK_CFG: mpsl_clock_lfclk_cfg_t = mpsl_clock_lfclk_cfg_t {
        source: MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };

    fn build_sdc<'d, const N: usize>(
        p: nrf_sdc::Peripherals<'d>,
        rng: &'d mut rng::Rng<RNG, Async>,
        mpsl: &'d MultiprotocolServiceLayer,
        mem: &'d mut sdc::Mem<N>,
    ) -> Result<SoftdeviceController<'d>, nrf_sdc::Error> {
        sdc::Builder::new()?.support_adv()?.build(p, rng, mpsl, mem)
    }
    pub(crate) fn new(
        ppi_ch17: Peri<'a, peripherals::PPI_CH17>,
        ppi_ch18: Peri<'a, peripherals::PPI_CH18>,
        ppi_ch19: Peri<'a, peripherals::PPI_CH19>,
        ppi_ch20: Peri<'a, peripherals::PPI_CH20>,
        ppi_ch21: Peri<'a, peripherals::PPI_CH21>,
        ppi_ch22: Peri<'a, peripherals::PPI_CH22>,
        ppi_ch23: Peri<'a, peripherals::PPI_CH23>,
        ppi_ch24: Peri<'a, peripherals::PPI_CH24>,
        ppi_ch25: Peri<'a, peripherals::PPI_CH25>,
        ppi_ch26: Peri<'a, peripherals::PPI_CH26>,
        ppi_ch27: Peri<'a, peripherals::PPI_CH27>,
        ppi_ch28: Peri<'a, peripherals::PPI_CH28>,
        ppi_ch29: Peri<'a, peripherals::PPI_CH29>,
        ppi_ch30: Peri<'a, peripherals::PPI_CH30>,
        ppi_ch31: Peri<'a, peripherals::PPI_CH31>,
        rng_p: Peri<'a, peripherals::RNG>,
        rtc0: Peri<'a, peripherals::RTC0>,
        timer0: Peri<'a, peripherals::TIMER0>,
        temp: Peri<'a, peripherals::TEMP>,
    ) -> Self {
        let sdc_p = sdc_Peripherals::new(
            ppi_ch17, ppi_ch18, ppi_ch20, ppi_ch21, ppi_ch22, ppi_ch23, ppi_ch24, ppi_ch25,
            ppi_ch26, ppi_ch27, ppi_ch28, ppi_ch29,
        );

        let sdc_mem = sdc::Mem::<SDC_MEMORY_SIZE>::new();

        Self {
            sdc_p,
            sdc_mem,
            ppi_ch19,
            ppi_ch30,
            ppi_ch31,
            rng_p,
            rtc0,
            timer0,
            temp,
        }
    }

    pub(crate) fn init(
        self,
    ) -> Result<
        (
            SoftdeviceController<'a>,
            &'static MultiprotocolServiceLayer<'a>,
        ),
        nrf_sdc::Error,
    > {
        let mpsl = {
            let p = mpsl_Peripherals::new(
                self.rtc0,
                self.timer0,
                self.temp,
                self.ppi_ch19,
                self.ppi_ch30,
                self.ppi_ch31,
            );
            MultiprotocolServiceLayer::new(p, Irqs, Self::LFCLK_CFG)
        }?;

        let sdc_rng = {
            static SDC_RNG: StaticCell<rng::Rng<'static, RNG, Async>> = StaticCell::new();
            SDC_RNG.init(rng::Rng::new(self.rng_p, Irqs))
        };

        let sdc_mem = {
            static SDC_MEM: StaticCell<sdc::Mem<SDC_MEMORY_SIZE>> = StaticCell::new();
            SDC_MEM.init(self.sdc_mem)
        };

        let mpsl = {
            static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
            MPSL.init(mpsl)
        };
        let sdc = Self::build_sdc(self.sdc_p, sdc_rng, mpsl, sdc_mem)?;

        Ok((sdc, mpsl))
    }
}

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
pub async fn sdc_task(sdc: &'static SoftdeviceController<'static>) -> ! {
    loop {}
}
