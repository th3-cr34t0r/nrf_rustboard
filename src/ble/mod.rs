#[cfg(feature = "defmt")]
use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::mode::Async;

use embassy_nrf::pac::FICR;
use embassy_nrf::peripherals::RNG;
use embassy_nrf::saadc;
use embassy_nrf::{bind_interrupts, qspi, rng};
use nrf_mpsl::raw::{
    MPSL_CLOCK_LF_SRC_RC, MPSL_DEFAULT_CLOCK_ACCURACY_PPM, MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED,
    MPSL_RECOMMENDED_RC_CTIV, MPSL_RECOMMENDED_RC_TEMP_CTIV,
};
use nrf_mpsl::{Flash, Peripherals as mpsl_Peripherals};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{
    self as sdc, Peripherals as sdc_Peripherals, SoftdeviceController,
    mpsl::{
        ClockInterruptHandler, HighPrioInterruptHandler, LowPrioInterruptHandler, SessionMem,
        raw::mpsl_clock_lfclk_cfg_t,
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use static_cell::StaticCell;
use trouble_host::Address;
use trouble_host::prelude::{DefaultPacketPool, Runner};

use crate::peripherals::BlePeri;

#[cfg(feature = "central")]
mod central;
#[cfg(feature = "peripheral")]
mod peripheral;
mod services;

bind_interrupts!(pub struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => LowPrioInterruptHandler;
    CLOCK_POWER => ClockInterruptHandler;
    RADIO => HighPrioInterruptHandler;
    TIMER0 => HighPrioInterruptHandler;
    RTC0 => HighPrioInterruptHandler;
    QSPI => qspi::InterruptHandler<embassy_nrf::peripherals::QSPI>;
    SAADC => saadc::InterruptHandler;
});

/// How many outgoing L2CAP buffers per link
const L2CAP_TXQ: u8 = 3;

/// How many incoming L2CAP buffers per link
const L2CAP_RXQ: u8 = 3;

/// Size of L2CAP packets
const L2CAP_MTU: usize = 251;

#[cfg(feature = "central")]
/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 2816; // bytes
#[cfg(feature = "peripheral")]
/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 5080; // bytes

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

/// Background ble task
pub async fn ble_task(
    mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>,
) {
    #[cfg(feature = "defmt")]
    info!("[ble_task] running runner");
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] error: {:?}", e);
        }
    }
}

const LFCLK_CFG: mpsl_clock_lfclk_cfg_t = mpsl_clock_lfclk_cfg_t {
    source: MPSL_CLOCK_LF_SRC_RC as u8,
    rc_ctiv: MPSL_RECOMMENDED_RC_CTIV as u8,
    rc_temp_ctiv: MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
    accuracy_ppm: MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
    skip_wait_lfclk_started: MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
};

/// Build SoftDevice
fn build_sdc<'a, const N: usize>(
    p: nrf_sdc::Peripherals<'a>,
    rng: &'a mut rng::Rng<Async>,
    mpsl: &'a MultiprotocolServiceLayer,
    mem: &'a mut sdc::Mem<N>,
) -> Result<SoftdeviceController<'a>, nrf_sdc::Error> {
    if cfg!(feature = "central") {
        sdc::Builder::new()?
            .support_scan()?
            .support_central()?
            .support_le_2m_phy()?
            .support_phy_update_central()?
            .central_count(1)?
            .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
            .build(p, rng, mpsl, mem)
    } else {
        sdc::Builder::new()?
            .support_adv()?
            .support_peripheral()?
            .support_le_2m_phy()?
            .support_phy_update_peripheral()?
            .peripheral_count(2)?
            .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
            .build(p, rng, mpsl, mem)
    }
}
pub async fn ble_init_run(ble_peri: BlePeri, spawner: Spawner) {
    let sdc_p = sdc_Peripherals::new(
        ble_peri.ppi_ch17,
        ble_peri.ppi_ch18,
        ble_peri.ppi_ch20,
        ble_peri.ppi_ch21,
        ble_peri.ppi_ch22,
        ble_peri.ppi_ch23,
        ble_peri.ppi_ch24,
        ble_peri.ppi_ch25,
        ble_peri.ppi_ch26,
        ble_peri.ppi_ch27,
        ble_peri.ppi_ch28,
        ble_peri.ppi_ch29,
    );

    let sdc_mem = sdc::Mem::<SDC_MEMORY_SIZE>::new();

    let mpsl = {
        let mpsl_peri = mpsl_Peripherals::new(
            ble_peri.rtc0,
            ble_peri.timer0,
            ble_peri.temp,
            ble_peri.ppi_ch19,
            ble_peri.ppi_ch30,
            ble_peri.ppi_ch31,
        );
        static SESSION_MEM: StaticCell<SessionMem<1>> = StaticCell::new();

        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        MPSL.init(
            MultiprotocolServiceLayer::with_timeslots(
                mpsl_peri,
                Irqs,
                LFCLK_CFG,
                SESSION_MEM.init(SessionMem::new()),
            )
            .expect("[ble] Error initializing MPSL"),
        )
    };

    // Use internal Flash as storage
    let mut storage = Flash::take(mpsl, ble_peri.nvmc);

    let mut sdc_rng = {
        static SDC_RNG: StaticCell<rng::Rng<'static, Async>> = StaticCell::new();
        SDC_RNG.init(rng::Rng::new(ble_peri.rng, Irqs))
    };

    let sdc_mem = {
        static SDC_MEM: StaticCell<sdc::Mem<SDC_MEMORY_SIZE>> = StaticCell::new();
        SDC_MEM.init(sdc_mem)
    };

    let mut rng = ChaCha12Rng::from_rng(&mut sdc_rng).unwrap();

    let sdc = build_sdc(sdc_p, sdc_rng, mpsl, sdc_mem).expect("[ble] Error building SDC");

    // run the mpsl task
    spawner.must_spawn(mpsl_task(mpsl));

    #[cfg(feature = "central")]
    crate::ble::central::ble_central_run(
        sdc,
        &mut storage,
        &mut rng,
        ble_peri.p_04,
        ble_peri.saadc,
    )
    .await;
    #[cfg(feature = "peripheral")]
    crate::ble::peripheral::ble_peripheral_run(
        sdc,
        &mut storage,
        &mut rng,
        ble_peri.p_04,
        ble_peri.saadc,
    )
    .await;
}

pub fn get_device_address() -> Address {
    let addr_0 = FICR.deviceid(0).read();
    let addr_1 = FICR.deviceid(1).read();

    let high = u64::from(addr_0);
    let addr = (high << 32) | u64::from(addr_1);
    let addr = addr | 0x0000_c000_0000_0000;

    Address::random(
        addr.to_le_bytes()[..6]
            .try_into()
            .expect("[addr] issue getting ble address"),
    )
}
