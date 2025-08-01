use serde::Serialize;
use trouble_host::prelude::{characteristic::BATTERY_LEVEL, *};
use usbd_hid::descriptor::{SerializedDescriptor, gen_hid_descriptor, generator_prelude::*};

pub const HUMAN_INTERFACE_DEVICE: BluetoothUuid16 = BluetoothUuid16::new(0x1812);
pub const BATTERY: BluetoothUuid16 = BluetoothUuid16::new(0x180f);

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
    #[characteristic(uuid = "2a4b", read, value = KeyboardReport::desc().try_into().expect("Failed to convert keyboard report to [u8; 67]"))]
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
/// KeyboardReport describes a report and its companion descriptor that can be
/// used to send keyboard button presses to a host and receive the status of the
/// keyboard LEDs.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (logical_min = 0x00,) = {
            #[item_settings constant,variable,absolute] reserved=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
#[allow(dead_code)]
#[derive(defmt::Format)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}
