use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::select3;

use embedded_storage_async::nor_flash::NorFlash;
use nrf_sdc::Error;
use nrf_sdc::SoftdeviceController;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use rand::{CryptoRng, RngCore};
use static_cell::StaticCell;
use trouble_host::att::AttErrorCode;
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::Runner;
use trouble_host::prelude::service::{BATTERY, HUMAN_INTERFACE_DEVICE};
use trouble_host::prelude::{
    AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
    Peripheral, appearance,
};
use trouble_host::{Address, BleHostError, Host, Stack};

use crate::ble::BleHostResources;
use crate::ble::get_device_address;
use crate::config::BLE_NAME;
use crate::storage::{load_bonding_info, store_bonding_info};

use ssmarshal::{self, serialize};

use crate::ble::services::Server;
use crate::{KEY_REPORT, delay_ms};

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn host_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.expect("Host task failed to run");
}

/// run ble
pub async fn ble_peripheral_run<RNG, S>(
    sdc: SoftdeviceController<'static>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    mut storage: &mut S,
    rng: &mut RNG,
    spawner: Spawner,
) where
    RNG: RngCore + CryptoRng,
    S: NorFlash,
{
    // ble address
    let address: Address = get_device_address();
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

    // run the mpsl task
    spawner.must_spawn(mpsl_task(&mpsl));

    // run the host task
    spawner.must_spawn(host_task(runner));

    // advertiser
    loop {
        match advertise(&mut peripheral, &server).await {
            Ok(conn) => {
                // set bondable
                conn.raw()
                    .set_bondable(!bond_stored)
                    .expect("[ble] error setting bondable");

                info!("[adv] bond_stored: {}", bond_stored);
                info!("[adv] Connected! Running service tasks");

                select3(
                    gatt_events_handler(&conn, &server, &mut storage, &mut bond_stored),
                    battery_service_task(&conn, &server),
                    keyboard_service_task(&conn, &server),
                )
                .await;
            }
            Err(e) => {
                error!("{}", e);
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
                BATTERY.to_le_bytes(),
                HUMAN_INTERFACE_DEVICE.to_le_bytes(),
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

async fn gatt_events_handler<'stack, 'server, S: NorFlash>(
    conn: &GattConnection<'stack, 'server, DefaultPacketPool>,
    server: &'server Server<'_>,
    storage: &mut S,
    bond_stored: &mut bool,
) -> Result<(), Error> {
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
                        if event.handle() == hid_service.handle {
                            let value = server.get(&hid_service);
                            info!("[gatt] Read Event to HID Characteristic: {:?}", value);
                        } else if event.handle() == battery_service.handle {
                            let value = server.get(&battery_service);
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
async fn keyboard_service_task<'stack, 'server>(
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
