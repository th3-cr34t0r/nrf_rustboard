#![no_std]
#![no_main]

use core::{
    cell::{Cell, RefCell, RefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{Either, select},
};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{self, USBD},
    usb::{
        Driver, InterruptHandler,
        vbus_detect::{self, HardwareVbusDetect},
    },
};
use embassy_sync::{self, blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use embassy_usb::{
    self, Builder, Config, Handler, UsbDevice,
    class::hid::{
        Config as ClassConfig, HidReader, HidReaderWriter, HidWriter, ReportId, RequestHandler,
        State as HidState,
    },
    control::OutResponse,
};
use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
   USBD => InterruptHandler<peripherals::USBD>;
   CLOCK_POWER => vbus_detect::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let usb_driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let usb = usb_builder(usb_driver);

    let mut hid_state = HidState::new();
    let mut btn = Input::new(p.P0_17, Pull::Up);
    let mut led = Output::new(p.P0_15, Level::Low, OutputDrive::Standard);

    // let mut usb_hid =
    // UsbBuilder::new(&mut build_descriptors, &mut device_handler, &mut hid_state).await;

    Timer::after_millis(5000).await;
    // usb_hid.run(&mut btn, &mut led).await;
}

// struct UsbBuilder<'a> {
//     usb: UsbDevice<'a, embassy_nrf::usb::Driver<'a, USBD, HardwareVbusDetect>>,
//     reader: HidReader<'a, Driver<'a, USBD, HardwareVbusDetect>, 1>,
//     writer: HidWriter<'a, Driver<'a, USBD, HardwareVbusDetect>, 8>,
//     remote_wakeup: Signal<CriticalSectionRawMutex, ()>,
// }
fn usb_builder<'a>(
    driver: Driver<'a, USBD, HardwareVbusDetect>,
) -> Builder<'a, Driver<'a, USBD, HardwareVbusDetect>> {
    // let config = ClassConfig {
    //     report_descriptor: KeyboardReport::desc(),
    //     request_handler: None,
    //     poll_ms: 10,
    //     max_packet_size: 64,
    // };

    // let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, state, config);
    // let (reader, writer) = hid.split();
    // let remote_wakeup: Signal<CriticalSectionRawMutex, ()> = Signal::new();

    // create embassy-usb config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("th3-cr34t0r");
    config.product = Some("Rustboard");
    config.serial_number = Some("123456");
    config.max_power = 100;
    config.supports_remote_wakeup = true;

    // for windows compatibility
    config.max_packet_size_0 = 64;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // create DeviceBuilder
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 16]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 16]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 16])[..],
        &mut MSOS_DESC.init([0; 16])[..],
        &mut CONTROL_BUF.init([0; 64])[..],
    );

    static DEVICE_HANDLER: StaticCell<DeviceHandler> = StaticCell::new();

    builder.handler(DEVICE_HANDLER.init(DeviceHandler::new()));

    builder
}

//     pub async fn run(&mut self, btn: &mut Input<'a>, led: &mut Output<'a>) {
//         // let mut request_handler = HidRequestHandler {};
//         //
//         led.set_high();

//         let usb_fut = async {
//             led.toggle();
//             loop {
//                 self.usb.run_until_suspend().await;
//                 match select(self.usb.wait_resume(), self.remote_wakeup.wait()).await {
//                     Either::First(_) => (),
//                     Either::Second(_) => self.usb.remote_wakeup().await.unwrap(),
//                 }
//             }
//         };

//         let in_fut = async {
//             // led.set_high();
//             btn.wait_for_low().await;
//             if SUSPENDED.load(Ordering::Acquire) {
//                 self.remote_wakeup.signal(());
//             } else {
//                 let report = KeyboardReport {
//                     keycodes: [4, 0, 0, 0, 0, 0],
//                     modifier: 0,
//                     reserved: 0,
//                     leds: 0,
//                 };

//                 match self.writer.write_serialize(&report).await {
//                     Ok(()) => (),
//                     Err(e) => warn!("{:?}", e),
//                 }
//             }

//             btn.wait_for_high().await;
//             // led.set_low();
//             let report = KeyboardReport {
//                 keycodes: [0, 0, 0, 0, 0, 0],
//                 modifier: 0,
//                 reserved: 0,
//                 leds: 0,
//             };

//             match self.writer.write_serialize(&report).await {
//                 Ok(()) => (),
//                 Err(e) => warn!("{:?}", e),
//             }
//         };

//         let out_fut = async {
//             // self.reader.run(false, &mut request_handler).await;
//         };

//         join(usb_fut, join(in_fut, out_fut)).await;
// }
// }

struct HidRequestHandler {}

impl RequestHandler for HidRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        // info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        // info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        // info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        // info!("Get idle rate for {:?}", id);
        None
    }
}

static USB_ENABLED: AtomicBool = AtomicBool::new(false);

struct DeviceHandler {
    configured: AtomicBool,
}
impl DeviceHandler {
    fn new() -> Self {
        Self {
            configured: AtomicBool::new(false),
        }
    }
}
impl Handler for DeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        USB_ENABLED.store(false, Ordering::Release);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            info!(
                "Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled)."
            );
            USB_ENABLED.store(true, Ordering::Release);
        } else {
            USB_ENABLED.store(false, Ordering::Release);
            if self.configured.load(Ordering::Relaxed) {
                info!(
                    "Device resumed, it may now draw up to the configured current limit from Vbus"
                );
            } else {
                info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}
