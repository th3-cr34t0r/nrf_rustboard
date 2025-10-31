use serde::Serialize;
use trouble_host::prelude::{
    characteristic::{BATTERY_LEVEL, BATTERY_LEVEL_STATUS},
    *,
};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

pub const HUMAN_INTERFACE_DEVICE: BluetoothUuid16 = BluetoothUuid16::new(0x1812);
pub const BATTERY: BluetoothUuid16 = BluetoothUuid16::new(0x180f);

#[gatt_server]
pub(crate) struct Server {
    pub(crate) battery_service: BatteryService,
    pub(crate) hid_service: HidService,
}

#[gatt_service(uuid = service::BATTERY)]
pub(crate) struct BatteryService {
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "hello", read, value = "Battery Level")]
    #[characteristic(uuid = BATTERY_LEVEL, read, notify, value = 10)]
    pub(crate) level: u8,
    #[characteristic(uuid = BATTERY_LEVEL_STATUS, write, read, notify)]
    status: bool,
}
#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub(crate) struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    pub(crate) hid_info: [u8; 4],
    #[characteristic(uuid = "2a4b", read, value = KeyboardReport::desc().try_into().expect("Failed to convert KeyboardReport to [u8; 69]"))]
    pub(crate) report_map: [u8; 69],
    #[characteristic(uuid = "2a4c", write_without_response)]
    pub(crate) hid_control_point: u8,
    #[characteristic(uuid = "2a4e", read, write_without_response, value = 1)]
    pub(crate) protocol_mode: u8,
    #[descriptor(uuid = "2908", read, value = [0u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify)]
    pub(crate) input_keyboard: [u8; 8],
    #[descriptor(uuid = "2908", read, value = [0u8, 2u8])]
    #[characteristic(uuid = "2a4d", read, write, write_without_response)]
    pub(crate) output_keyboard: [u8; 1],
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
