use ble_server::KeyboardReport;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{self, RNG};
use embassy_nrf::{Peri, bind_interrupts, qspi, rng};
use embassy_time::Timer;
use nrf_mpsl::raw::{
    MPSL_CLOCK_LF_SRC_RC, MPSL_DEFAULT_CLOCK_ACCURACY_PPM, MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED,
    MPSL_RECOMMENDED_RC_CTIV, MPSL_RECOMMENDED_RC_TEMP_CTIV,
};
use nrf_mpsl::{Flash, Peripherals as mpsl_Peripherals};
use nrf_sdc::Error;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{
    self as sdc, Mem, Peripherals as sdc_Peripherals, SoftdeviceController,
    mpsl::{
        ClockInterruptHandler, HighPrioInterruptHandler, LowPrioInterruptHandler, SessionMem,
        raw::mpsl_clock_lfclk_cfg_t,
    },
};
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use static_cell::StaticCell;
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::{
    AdStructure, Advertisement, AdvertisementParameters, AttributeHandle, BR_EDR_NOT_SUPPORTED,
    DefaultPacketPool, LE_GENERAL_DISCOVERABLE, Peripheral, Runner, appearance,
};
use trouble_host::{Address, BleHostError, Host, HostResources, IoCapabilities, Stack};

use ssmarshal;

use crate::ble::ble_server::{BleHidServer, Server};
mod ble_server;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => LowPrioInterruptHandler;
    CLOCK_POWER => ClockInterruptHandler;
    RADIO => HighPrioInterruptHandler;
    TIMER0 => HighPrioInterruptHandler;
    RTC0 => HighPrioInterruptHandler;
    QSPI => qspi::InterruptHandler<embassy_nrf::peripherals::QSPI>;
});

/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 5000; // bytes

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

    /// How many outgoing L2CAP buffers per link
    const L2CAP_TXQ: u8 = 3;

    /// How many incoming L2CAP buffers per link
    const L2CAP_RXQ: u8 = 3;

    /// Size of L2CAP packets
    const L2CAP_MTU: usize = 251;

    /// Build SoftDevice
    fn build_sdc<'d, const N: usize>(
        p: nrf_sdc::Peripherals<'d>,
        rng: &'d mut rng::Rng<Async>,
        mpsl: &'d MultiprotocolServiceLayer,
        mem: &'d mut sdc::Mem<N>,
    ) -> Result<SoftdeviceController<'d>, nrf_sdc::Error> {
        sdc::Builder::new()?
            .support_adv()?
            .support_peripheral()?
            .peripheral_count(1)?
            .buffer_cfg(
                Self::L2CAP_MTU as u16,
                Self::L2CAP_MTU as u16,
                Self::L2CAP_TXQ,
                Self::L2CAP_RXQ,
            )?
            .build(p, rng, mpsl, mem)
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
        static SESSION_MEM: StaticCell<SessionMem<1>> = StaticCell::new();

        let mpsl = {
            let p = mpsl_Peripherals::new(
                self.rtc0,
                self.timer0,
                self.temp,
                self.ppi_ch19,
                self.ppi_ch30,
                self.ppi_ch31,
            );

            MultiprotocolServiceLayer::with_timeslots(
                p,
                Irqs,
                Self::LFCLK_CFG,
                SESSION_MEM.init(SessionMem::new()),
            )
        }?;

        let mut sdc_rng = {
            static SDC_RNG: StaticCell<rng::Rng<'static, Async>> = StaticCell::new();
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
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;
const ADV_SETS: usize = 1;

type BleHostResources = HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>;

#[embassy_executor::task]
async fn host_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.expect("Host task failed to run");
}

async fn advertise<'a, 'b>(
    peripheral: &mut Peripheral<'a, SoftdeviceController<'static>, DefaultPacketPool>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<Error>> {
    let mut advertiser_data = [0; 31];

    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[
                ble_server::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                // ble_server::BATTERY.to_le_bytes(),
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

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;

    info!("[adv] Advertising; waiting for connection...");

    let conn = advertiser.accept().await?.with_attribute_server(server)?;

    info!("[adv] Connection established");

    Ok(conn)
}

async fn gatt_events_handler<'stack, 'server>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
) {
    let hid_service = server.hid_service.report_map;
    let battery_service = server.battery_service.level;

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::PairingComplete {
                security_level,
                bond,
            } => {
                info!("[gatt] pairing complete: {:?}", security_level);
            }
            GattConnectionEvent::PairingFailed(err) => {
                error!("[gatt] pairing error: {:?}", err);
            }
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == battery_service.handle {
                            let value = server.get(&battery_service);
                            info!("[gatt) Read Event to Level Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == battery_service.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic {:?}",
                                event.data()
                            );
                        }
                    }
                    GattEvent::Other(event) => {
                        info!("[gatt] GattEvent::OTHER");
                    }
                };

                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => {
                        error!("error sending response {:?}", e)
                    }
                };
            }
            _ => {} // ignore other Gatt connection events
        }
    };

    error!("Disconnected reason: {}", reason);
}

async fn custom_task<'stack, 'server>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
) {
    let battery_level = server.battery_service.level;

    let mut tick: u8 = 0;
    loop {
        tick = tick.wrapping_add(1);
        if battery_level.notify(conn, &tick).await.is_err() {
            break;
        }

        Timer::after_millis(1000).await
    }
}

/// Run BLE
pub async fn run<RNG>(
    softdevice_controller: SoftdeviceController<'static>,
    random_generator: &mut RNG,
    flash: Flash<'_>,
    spawner: Spawner,
) where
    RNG: RngCore + CryptoRng,
{
    // ble address
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Address: {}", address);

    let resources = {
        static RESOURCES: StaticCell<BleHostResources> = StaticCell::new();
        RESOURCES.init(BleHostResources::new())
    };

    let stack = {
        static STACK: StaticCell<Stack<'_, SoftdeviceController<'_>, DefaultPacketPool>> =
            StaticCell::new();
        STACK.init(
            trouble_host::new(softdevice_controller, resources)
                .set_random_address(address)
                .set_random_generator_seed(random_generator), // .set_io_capabilities(IoCapabilities::NoInputNoOutput), //suitable for a keyboard
        )
    };

    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    // let mut bond_stored = if let Some (bond_info) = load_bond

    // create the peripheral server
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: BLE_NAME,
        appearance: &appearance::human_interface_device::KEYBOARD,
    }))
    .expect("Failed to create GATT Server");

    // run the host task
    spawner.must_spawn(host_task(runner));

    // advertiser
    loop {
        match advertise(&mut peripheral, &server).await {
            Ok(conn) => {
                // set bondable
                conn.raw().set_bondable(true).unwrap();

                info!("[adv] Connected!");

                let gatt_events_task = gatt_events_handler(&conn, &server);
                let custom_task = custom_task(&conn, &server);

                select(gatt_events_task, custom_task).await;
            }
            Err(e) => {
                error!("{}", e);
                // Timer::after_millis(500).await;
            }
        }
    }
}
