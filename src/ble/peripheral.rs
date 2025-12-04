use defmt::{error, info};
use embassy_futures::join::join;
use embassy_futures::select::select3;

use embedded_storage_async::nor_flash::NorFlash;
use nrf_sdc::Error;
use nrf_sdc::SoftdeviceController;
use rand::{CryptoRng, RngCore};
use static_cell::StaticCell;
use trouble_host::HostResources;
use trouble_host::att::AttErrorCode;
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::service::{BATTERY, HUMAN_INTERFACE_DEVICE};
use trouble_host::prelude::{
    AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
    Peripheral, appearance,
};
use trouble_host::{Address, BleHostError, Host, Stack};

use crate::MATRIX_KEYS_SPLIT;
use crate::ble::ble_task;
use crate::ble::get_device_address;
use crate::ble::services::SPLIT_SERVICE;
use crate::config::BLE_NAME;
use crate::config::COLS;
use crate::config::MATRIX_KEYS_BUFFER;
use crate::matrix::KeyPos;
use crate::storage::{load_bonding_info, store_bonding_info};

use ssmarshal::{self, serialize};

use crate::ble::services::Server;
use crate::{KEY_REPORT, delay_ms};

const CONNECTIONS_MAX: usize = 2;

const L2CAP_CHANNELS_MAX: usize = CONNECTIONS_MAX * 4;

type BleHostResources = HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>;

/// run ble
pub async fn ble_peripheral_run<RNG, S>(
    sdc: SoftdeviceController<'static>,
    // mpsl: &'static MultiprotocolServiceLayer<'static>,
    mut storage: &mut S,
    rng: &mut RNG,
) where
    RNG: RngCore + CryptoRng,
    S: NorFlash,
{
    // ble address
    let address: Address = get_device_address();

    #[cfg(feature = "debug")]
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
                .set_random_generator_seed(rng),
        )
    };

    // get the bond information
    let mut bond_stored = if let Some(bond_info) = load_bonding_info(storage).await {
        stack.add_bond_information(bond_info).unwrap();
        info!("[ble] loaded bond information");
        true
    } else {
        info!("[ble] no bond information found");
        false
    };

    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    // create the peripheral server
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: BLE_NAME,
        appearance: &appearance::human_interface_device::KEYBOARD,
    }))
    .expect("Failed to create GATT Server");

    let _ = join(
        // backgroun task
        ble_task(runner),
        // advertiser
        async {
            loop {
                match advertise(&mut peripheral, &server).await {
                    Ok(conn_1) => {
                        // info!("[adv] bond_stored: {}", bond_stored);
                        info!("[adv] Connected! Running service tasks");
                        delay_ms(1000).await;

                        let _ = join(gatt_events_handler(&conn_1, &server), async {
                            loop {
                                // advertise to connect second central
                                match advertise(&mut peripheral, &server).await {
                                    Ok(conn_2) => {
                                        // set bondable
                                        conn_2
                                            .raw()
                                            .set_bondable(!bond_stored)
                                            .expect("[ble] error setting bondable");

                                        let _ = select3(
                                            gatt_events_handler_2(
                                                &conn_2,
                                                &server,
                                                &mut storage,
                                                &mut bond_stored,
                                            ),
                                            battery_service_task(&conn_2, &server),
                                            hid_kb_service_task(&conn_2, &server),
                                        )
                                        .await;
                                    }
                                    Err(e) => {
                                        error!("{}", e);
                                        delay_ms(1000).await;
                                    }
                                }
                            }
                        })
                        .await;
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        },
    )
    .await;
}

/// Advertiser task
async fn advertise<'a, 'b>(
    peripheral: &mut Peripheral<'a, SoftdeviceController<'static>, DefaultPacketPool>,
    server: &'b Server<'_>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<Error>> {
    let mut advertiser_data = [0; 31];

    #[cfg(feature = "debug")]
    info!("[adv] creating adStructure");

    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[
                BATTERY.to_le_bytes(),
                HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                SPLIT_SERVICE.to_le_bytes(),
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

    #[cfg(feature = "debug")]
    info!("[adv] creating advertiser");

    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        )
        .await?;

    #[cfg(feature = "debug")]
    info!("[adv] advertising, waiting for connection...");

    let gatt_conn = advertiser.accept().await?.with_attribute_server(&server)?;

    info!("[adv] connection established");

    Ok(gatt_conn)
}

/// Gatt event handelr task
async fn gatt_events_handler<'stack, 'server>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
) -> Result<(), Error> {
    let hid_service_report_map = server.hid_service.report_map;
    let battery_service_level = server.battery_service.level;
    let split_service_registered_keys = server.split_service.registered_keys;

    let matrix_keys_split_sender = MATRIX_KEYS_SPLIT.sender();
    let mut matrix_keys_split_local = [KeyPos::default(); MATRIX_KEYS_BUFFER];

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::PairingComplete {
                security_level,
                bond,
            } => {
                info!("[gatt] pairing complete: {:?}", security_level);
                if let Some(bond) = bond {
                    // store_bonding_info(storage, &bond)
                    //     .await
                    //     .expect("[gatt] error storing bond info");
                    // *bond_stored = true;
                    info!("[gatt] bond information stored");
                }
            }
            GattConnectionEvent::PairingFailed(err) => {
                error!("[gatt] pairing error: {:?}", err);
            }
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == hid_service_report_map.handle {
                            let value = server.get(&hid_service_report_map);
                            info!("[gatt] Read Event to HID Characteristic: {:?}", value);
                        } else if event.handle() == battery_service_level.handle {
                            let value = server.get(&battery_service_level);
                            info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                        }

                        if conn
                            .raw()
                            .security_level()
                            .expect("[gatt] error getting security level")
                            .encrypted()
                        {
                            None
                        } else {
                            Some(AttErrorCode::INSUFFICIENT_ENCRYPTION)
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == split_service_registered_keys.handle {
                            // central message to peripheral
                            let central_data = event.data();

                            // store the central keys in matrix keys
                            for (index, combined_key) in central_data.iter().enumerate() {
                                if *combined_key != 255u8 {
                                    // if let Some(index) = matrix_keys_local
                                    //     .iter_mut()
                                    //     .position(|m_key| *m_key == KeyPos::default())
                                    // {
                                    let col = (combined_key & 0x0f) + COLS as u8;
                                    let row = combined_key >> 4;

                                    matrix_keys_split_local[index] = KeyPos { row, col };
                                } else {
                                    matrix_keys_split_local[index] = KeyPos::default();
                                }
                            }
                            // send the new matrix_keys
                            matrix_keys_split_sender.send(matrix_keys_split_local);
                        } else if event.handle() == hid_service_report_map.handle {
                            info!(
                                "[gatt] Write Event to HID Characteristic {:?}",
                                event.data()
                            );
                        } else if event.handle() == battery_service_level.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic {:?}",
                                event.data()
                            );
                        }

                        if conn
                            .raw()
                            .security_level()
                            .expect("[gatt] error getting security level")
                            .encrypted()
                        {
                            None
                        } else {
                            Some(AttErrorCode::INSUFFICIENT_ENCRYPTION)
                        }
                    }

                    _ => None, // OTHER
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
    Ok(())
}

/// Gatt event handelr task
async fn gatt_events_handler_2<'stack, 'server, S: NorFlash>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
    storage: &mut S,
    bond_stored: &mut bool,
) -> Result<(), Error> {
    let hid_service_report_map = server.hid_service.report_map;
    let battery_service_level = server.battery_service.level;
    let split_service_registered_keys = server.split_service.registered_keys;

    let matrix_keys_split_sender = MATRIX_KEYS_SPLIT.sender();
    let mut matrix_keys_split_local = [KeyPos::default(); MATRIX_KEYS_BUFFER];

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::PairingComplete {
                security_level,
                bond,
            } => {
                info!("[gatt] pairing complete: {:?}", security_level);
                if let Some(bond) = bond {
                    store_bonding_info(storage, &bond)
                        .await
                        .expect("[gatt] error storing bond info");
                    *bond_stored = true;
                    info!("[gatt] bond information stored");
                }
            }
            GattConnectionEvent::PairingFailed(err) => {
                error!("[gatt] pairing error: {:?}", err);
            }
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == hid_service_report_map.handle {
                            let value = server.get(&hid_service_report_map);
                            info!("[gatt] Read Event to HID Characteristic: {:?}", value);
                        } else if event.handle() == battery_service_level.handle {
                            let value = server.get(&battery_service_level);
                            info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                        }

                        if conn
                            .raw()
                            .security_level()
                            .expect("[gatt] error getting security level")
                            .encrypted()
                        {
                            None
                        } else {
                            Some(AttErrorCode::INSUFFICIENT_ENCRYPTION)
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == split_service_registered_keys.handle {
                            // central message to peripheral
                            let central_data = event.data();

                            // store the central keys in matrix keys
                            for (index, combined_key) in central_data.iter().enumerate() {
                                if *combined_key != 255u8 {
                                    let col = (combined_key & 0x0f) + COLS as u8;
                                    let row = combined_key >> 4;

                                    matrix_keys_split_local[index] = KeyPos { row, col };
                                } else {
                                    matrix_keys_split_local[index] = KeyPos::default();
                                }
                            }
                            // send the new matrix_keys
                            matrix_keys_split_sender.send(matrix_keys_split_local);
                        } else if event.handle() == hid_service_report_map.handle {
                            info!(
                                "[gatt] Write Event to HID Characteristic {:?}",
                                event.data()
                            );
                        } else if event.handle() == battery_service_level.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic {:?}",
                                event.data()
                            );
                        }

                        if conn
                            .raw()
                            .security_level()
                            .expect("[gatt] error getting security level")
                            .encrypted()
                        {
                            None
                        } else {
                            Some(AttErrorCode::INSUFFICIENT_ENCRYPTION)
                        }
                    }

                    _ => None, // OTHER
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
    Ok(())
}

/// Battery service task
async fn battery_service_task<'stack, 'server>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
) {
    let battery_characteristic = server.battery_service.level;

    let mut tick: u8 = 0;

    loop {
        tick = tick.wrapping_add(1);
        match battery_characteristic.notify(conn, &tick).await {
            Ok(_) => {
                #[cfg(feature = "debug")]
                info!("[notify] battery level notified successfully");
            }
            Err(e) => {
                info!("[notify] battery level error: {}", e);
                break;
            }
        }

        // send notification every 1 minute
        delay_ms(60000).await;
    }
}

/// Keyboard serivce task
async fn hid_kb_service_task<'stack, 'server>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
) {
    let mut buff = [0u8; 8];

    let mut key_report = KEY_REPORT
        .receiver()
        .expect(" [ble_peripheral] maximum number of receivers has been reached");

    loop {
        // wait till new key_report is received from key_provision
        let key_report = key_report.changed().await;

        let _n = serialize(&mut buff, &key_report).unwrap();

        match server.hid_service.input_keyboard.notify(conn, &buff).await {
            Ok(_) => {
                #[cfg(feature = "debug")]
                info!("[notify] input keyboard notified successfully")
            }
            Err(e) => {
                info!("[notify] input keyboard error: {}", e);
                break;
            }
        }
    }
}
