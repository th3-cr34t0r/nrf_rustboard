use trouble_host::prelude::characteristic::BATTERY_LEVEL;
use trouble_host::prelude::*;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

pub const HUMAN_INTERFACE_DEVICE: BluetoothUuid16 = BluetoothUuid16::new(0x1812);

#[gatt_server]
pub struct Server {
    battery_service: BatteryService,
    hid_service: HidService,
}

#[gatt_service(uuid = service::BATTERY)]
pub struct BatteryService {
    #[characteristic(uuid = BATTERY_LEVEL, read, notify)]
    battery_leves: u8,
}
#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub struct HidService {
    #[characteristic(uuid = "2a4b", read, value = KeyboardReport::desc().try_into().unwrap())]
    report_map: [u8; 67],
    #[characteristic(uuid = "2a22", read, notify)]
    input_keyboard: [u8; 8],
    #[characteristic(uuid = "2a32", read, write, write_without_response)]
    output_keyboard: [u8; 1],
}

pub struct BleHidServer {
    pub input_keyboard: Characteristic<[u8; 8]>,
}

impl<'s> BleHidServer {
    pub fn new(server: Server<'s>) -> Self {
        Self {
            input_keyboard: server.hid_service.input_keyboard,
        }
    }
}
