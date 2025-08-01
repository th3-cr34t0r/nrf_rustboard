use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{self, RNG};
use embassy_nrf::{Peri, bind_interrupts, rng};
use embassy_time::{Duration, Timer};
use nrf_mpsl::Peripherals as mpsl_Peripherals;
use nrf_mpsl::raw::{
    MPSL_CLOCK_LF_SRC_RC, MPSL_DEFAULT_CLOCK_ACCURACY_PPM, MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED,
    MPSL_RECOMMENDED_RC_CTIV, MPSL_RECOMMENDED_RC_TEMP_CTIV,
};
use nrf_sdc::Error;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{
    self as sdc, Mem, Peripherals as sdc_Peripherals, SoftdeviceController,
    mpsl::{
        ClockInterruptHandler, HighPrioInterruptHandler, LowPrioInterruptHandler,
        raw::mpsl_clock_lfclk_cfg_t,
    },
};
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use static_cell::StaticCell;
use trouble_host::gap::{CentralConfig, GapConfig};
use trouble_host::gatt::GattConnection;
use trouble_host::prelude::{
    AdStructure, Advertisement, AdvertisementParameters, BR_EDR_NOT_SUPPORTED, Central,
    DefaultPacketPool, LE_GENERAL_DISCOVERABLE, Peripheral, PhyKind, Runner, TxPower, appearance,
};
use trouble_host::{
    Address, BleHostError, Controller, Host, HostResources, PacketPool, Stack, central,
};

use crate::ble::ble_server::{BleHidServer, Server};
mod ble_server;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => LowPrioInterruptHandler;
    CLOCK_POWER => ClockInterruptHandler;
    RADIO => HighPrioInterruptHandler;
    TIMER0 => HighPrioInterruptHandler;
    RTC0 => HighPrioInterruptHandler;
});

/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 1120; // bytes

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
            ChaCha12Rng,
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

        let mut sdc_rng = {
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
        let rng = ChaCha12Rng::from_rng(&mut sdc_rng).unwrap();
        let sdc = Self::build_sdc(self.sdc_p, sdc_rng, mpsl, sdc_mem)?;

        Ok((sdc, mpsl, rng))
    }
}

const BLE_NAME: &str = "nRFRustboard";
const CONNS: usize = 2;
const CHANNELS: usize = 1;
const ADV_SETS: usize = 1;

type BleHostResources = HostResources<DefaultPacketPool, CONNS, CHANNELS, ADV_SETS>;

/// Run BLE
pub async fn run<RNG>(
    sdc: SoftdeviceController<'static>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    random_generator: &mut RNG,
    spawner: Spawner,
) where
    RNG: RngCore + CryptoRng,
{
    // ble address
    // let address = Address::random([0xff, 0x16, 0x56, 0x8f, 0x24, 0xff]);
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    // let address =Address::random( [0x18, 0xe2, 0x21, 0x80, 0xc0, 0xc7]);

    let resources = {
        static RESOURCES: StaticCell<BleHostResources> = StaticCell::new();
        RESOURCES.init(BleHostResources::new())
    };

    let stack = {
        static STACK: StaticCell<Stack<'_, SoftdeviceController<'_>, DefaultPacketPool>> =
            StaticCell::new();
        STACK.init(
            trouble_host::new(sdc, resources)
                .set_random_address(address)
                .set_random_generator_seed(random_generator),
        )
    };

    let Host {
        mut central,
        mut peripheral,
        runner,
        ..
    } = stack.build();

    // run the mpsl task
    spawner.must_spawn(mpsl_task(mpsl));
    // run the host task
    spawner.must_spawn(host_task(runner));

    let server = Server::new_with_config(GapConfig::Central(CentralConfig {
        name: BLE_NAME,
        appearance: &appearance::human_interface_device::KEYBOARD,
    }))
    .expect("Failed to create GATT Server");

    // let service = BleHidServer::new(server);

    // advertiser
    loop {
        match advertise(&mut peripheral, &server).await {
            Ok(connection) => info!("Connected"),
            Err(e) => {
                error!("{}", e);
                Timer::after_millis(500).await;
            }
        }
    }
}

async fn advertise<'a, 'b>(
    peripheral: &mut Peripheral<'a, SoftdeviceController<'static>, DefaultPacketPool>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<Error>> {
    let mut advertiser_data = [0; 31];

    let ad_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[
                ble_server::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                ble_server::BATTERY.to_le_bytes(),
            ]),
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),
            AdStructure::Unknown {
                ty: 0x19,
                data: &trouble_host::prelude::appearance::human_interface_device::KEYBOARD
                    .to_le_bytes(),
            },
        ],
        &mut advertiser_data[..],
    )?;

    info!("[debug] ad_len: {}", ad_len);

    info!("[debug] advertise_config");
    let advertise_config = AdvertisementParameters {
        primary_phy: PhyKind::Le2M,
        secondary_phy: PhyKind::Le2M,
        tx_power: TxPower::Plus8dBm,
        interval_min: Duration::from_millis(200),
        interval_max: Duration::from_millis(200),
        ..Default::default()
    };

    info!("[debug] advertiser");
    let advertiser = peripheral
        .advertise(
            &advertise_config,
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;

    info!("[debug] connection");
    let connection = advertiser.accept().await?.with_attribute_server(server)?;
    Ok(connection)
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn host_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.expect("Host task failed to run");
}
