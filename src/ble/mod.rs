use ble_server::KeyboardReport;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_nrf::mode::Async;

use embassy_nrf::peripherals::{
    self, NVMC, PPI_CH17, PPI_CH18, PPI_CH19, PPI_CH20, PPI_CH21, PPI_CH22, PPI_CH23, PPI_CH24,
    PPI_CH25, PPI_CH26, PPI_CH27, PPI_CH28, PPI_CH29, PPI_CH30, PPI_CH31, RNG, RTC0, TEMP, TIMER0,
};
use embassy_nrf::{Peri, Peripherals, bind_interrupts, qspi, rng};
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

const BLE_NAME: &str = "nRFRustboard";
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;
const ADV_SETS: usize = 1;

type BleHostResources = HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>;

/// Default memory allocation for softdevice controller in bytes.
const SDC_MEMORY_SIZE: usize = 5000; // bytes

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
fn build_sdc<'a, const N: usize>(
    p: nrf_sdc::Peripherals<'a>,
    rng: &'a mut rng::Rng<Async>,
    mpsl: &'a MultiprotocolServiceLayer,
    mem: &'a mut sdc::Mem<N>,
) -> Result<SoftdeviceController<'a>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()?
        .support_peripheral()?
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn host_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.expect("Host task failed to run");
}

pub fn ble_init(
    ppi_ch17: Peri<'static, PPI_CH17>,
    ppi_ch18: Peri<'static, PPI_CH18>,
    ppi_ch19: Peri<'static, PPI_CH19>,
    ppi_ch20: Peri<'static, PPI_CH20>,
    ppi_ch21: Peri<'static, PPI_CH21>,
    ppi_ch22: Peri<'static, PPI_CH22>,
    ppi_ch23: Peri<'static, PPI_CH23>,
    ppi_ch24: Peri<'static, PPI_CH24>,
    ppi_ch25: Peri<'static, PPI_CH25>,
    ppi_ch26: Peri<'static, PPI_CH26>,
    ppi_ch27: Peri<'static, PPI_CH27>,
    ppi_ch28: Peri<'static, PPI_CH28>,
    ppi_ch29: Peri<'static, PPI_CH29>,
    ppi_ch30: Peri<'static, PPI_CH30>,
    ppi_ch31: Peri<'static, PPI_CH31>,
    rtc0: Peri<'static, RTC0>,
    timer0: Peri<'static, TIMER0>,
    temp: Peri<'static, TEMP>,
    nvmc: Peri<'static, NVMC>,
    rng: Peri<'static, RNG>,
) -> Result<
    (
        SoftdeviceController<'static>,
        &'static MultiprotocolServiceLayer<'static>,
        Flash<'static>,
        ChaCha12Rng,
    ),
    nrf_sdc::Error,
> {
    let sdc_p = sdc_Peripherals::new(
        ppi_ch17, ppi_ch18, ppi_ch20, ppi_ch21, ppi_ch22, ppi_ch23, ppi_ch24, ppi_ch25, ppi_ch26,
        ppi_ch27, ppi_ch28, ppi_ch29,
    );

    let sdc_mem = sdc::Mem::<SDC_MEMORY_SIZE>::new();

    let mpsl = {
        let mpsl_peri = mpsl_Peripherals::new(rtc0, timer0, temp, ppi_ch19, ppi_ch30, ppi_ch31);
        static SESSION_MEM: StaticCell<SessionMem<1>> = StaticCell::new();

        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        MPSL.init(MultiprotocolServiceLayer::with_timeslots(
            mpsl_peri,
            Irqs,
            LFCLK_CFG,
            SESSION_MEM.init(SessionMem::new()),
        )?)
    };

    // Use internal Flash as storage
    let storage = Flash::take(mpsl, nvmc);

    let mut sdc_rng = {
        static SDC_RNG: StaticCell<rng::Rng<'static, Async>> = StaticCell::new();
        SDC_RNG.init(rng::Rng::new(rng, Irqs))
    };

    let sdc_mem = {
        static SDC_MEM: StaticCell<sdc::Mem<SDC_MEMORY_SIZE>> = StaticCell::new();
        SDC_MEM.init(sdc_mem)
    };

    let rng = ChaCha12Rng::from_rng(&mut sdc_rng).unwrap();

    let sdc = build_sdc(sdc_p, sdc_rng, mpsl, sdc_mem)?;

    Ok((sdc, mpsl, storage, rng))
}

/// Run BLE
pub async fn run<RNG>(
    sdc: SoftdeviceController<'static>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    storage: Flash<'static>,
    rng: &mut RNG,
    spawner: Spawner,
) where
    RNG: RngCore + CryptoRng,
{
    // ble address
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("[ble] addrress: {}", address);

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
                .set_random_generator_seed(rng), // .set_io_capabilities(IoCapabilities::NoInputNoOutput), //suitable for a keyboard
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

    // run the mpsl task
    spawner.must_spawn(mpsl_task(&mpsl));

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
    let hid_service = server.hid_service.report;
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
                        if event.handle() == hid_service.handle {
                            let value = server.get(&hid_service);
                            info!("[gatt) Read Event to HID Characteristic: {:?}", value);
                        } else if event.handle() == battery_service.handle {
                            let value = server.get(&battery_service);
                            info!("[gatt) Read Event to Level Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == hid_service.handle {
                            info!(
                                "[gatt] Write Event to HID Characteristic {:?}",
                                event.data()
                            );
                        } else if event.handle() == battery_service.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic {:?}",
                                event.data()
                            );
                        }
                    }
                    _ => {} // OTHER
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
    let battery_characteristic = server.battery_service.level;
    let hid_characteristic = server.hid_service.report;

    let mut tick: u8 = 0;

    let mut report: [u8; 67] = [0; 67];

    report[0] = 4;

    loop {
        tick = tick.wrapping_add(1);
        if battery_characteristic.notify(conn, &tick).await.is_err() {
            break;
        }

        // if hid_characteristic.notify(conn, &report).await.is_err() {
        //     break;
        // }

        Timer::after_millis(2000).await;
    }
}
