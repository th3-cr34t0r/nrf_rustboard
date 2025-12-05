// ble central
//

use defmt::info;
use embassy_futures::join::join;
use embassy_time::Duration;
use embedded_storage_async::nor_flash::NorFlash;
use nrf_sdc::{Error, SoftdeviceController};
use rand::{CryptoRng, RngCore};
use static_cell::StaticCell;
use trouble_host::{
    Address, Host, HostResources, Stack,
    gatt::GattClient,
    prelude::{
        Central, Characteristic, ConnectConfig, ConnectParams, Connection, DefaultPacketPool,
        ScanConfig, Uuid,
    },
};

use crate::{
    MESSAGE_TO_PERI,
    ble::{ble_task, get_device_address},
    config::PERI_ADDRESS,
    delay_ms,
};

const CONNECTIONS_MAX: usize = 1;

const L2CAP_CHANNELS_MAX: usize = CONNECTIONS_MAX + 4;

type BleHostResources = HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>;

/// run ble
pub async fn ble_central_run<RNG, S>(
    sdc: SoftdeviceController<'static>,
    mut storage: &mut S,
    rng: &mut RNG,
) where
    RNG: RngCore + CryptoRng,
    S: NorFlash,
{
    let address = get_device_address();

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

    let Host {
        mut central,
        runner,
        ..
    } = stack.build();

    let _ = join(ble_task(runner), async {
        loop {
            match connect(&mut central).await {
                Ok(conn) => {
                    // TODO: allow bonding

                    info!("[ble_connect] connected to peripheral");

                    // create client
                    let client = {
                        static CLIENT: StaticCell<
                            GattClient<'_, SoftdeviceController<'_>, DefaultPacketPool, 10>,
                        > = StaticCell::new();
                        CLIENT.init(
                            GattClient::<SoftdeviceController, DefaultPacketPool, 10>::new(
                                stack, &conn,
                            )
                            .await
                            .expect("[ble_central] error creating client"),
                        )
                    };

                    let _ = join(client.task(), split_keyboard_task(client)).await;
                }
                Err(e) => {
                    info!("[ble_central] error: {}", e);
                    break;
                }
            }
        }
    })
    .await;
}

async fn connect<'a, 'b>(
    central: &mut Central<'a, SoftdeviceController<'b>, DefaultPacketPool>,
) -> Result<Connection<'a, DefaultPacketPool>, Error> {
    // address of the target split kb
    let target = Address::random(PERI_ADDRESS);

    let conn_params = ConnectParams {
        min_connection_interval: Duration::from_micros(7500),
        max_connection_interval: Duration::from_micros(7500),
        max_latency: 0,
        min_event_length: Duration::from_secs(0),
        max_event_length: Duration::from_secs(0),
        supervision_timeout: Duration::from_secs(5),
    };

    let config = ConnectConfig {
        scan_config: ScanConfig {
            filter_accept_list: &[(target.kind, &target.addr)],
            ..Default::default()
        },
        connect_params: conn_params,
    };

    // Connect to peripheral
    info!("[ble_connect] connecting to peripheral {}", target);
    loop {
        // match select(delay_ms(5000), central.connect(&config)).await {
        //     Either::First(_) => {
        //         // if not connected, try again
        //         info!("[ble_connect] connect timeout");
        //         delay_ms(100).await;
        //     }
        //     Either::Second(e) => {
        //         match e {
        //             Ok(conn) => return Ok(conn),
        //             Err(e) => {
        //                 // error connecting
        //                 info!("[ble_connect] error connecting: {}", e);
        //                 delay_ms(100).await;
        //             }
        //         }
        //     }
        // }
        match central.connect(&config).await {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                // error connecting
                info!("[ble_connect] error connecting: {}", e);
                delay_ms(100).await;
            }
        }
    }
}

async fn split_keyboard_task<'a>(
    client: &'a GattClient<'a, SoftdeviceController<'a>, DefaultPacketPool, 10>,
) {
    let services = client
        .services_by_uuid(&Uuid::new_short(0xff11))
        .await
        .expect("[ble_central] unable to set services");

    let service = services.first().unwrap().clone();

    let characteristic: Characteristic<[u8; 6]> = client
        .characteristic_by_uuid(&service, &Uuid::new_short(0xff22))
        .await
        .expect("[ble_central] unable to set characteristic");

    let mut message_to_peri = MESSAGE_TO_PERI
        .receiver()
        .expect(" [ble_peripheral] maximum number of receivers has been reached");

    info!("[ble_split_keyboard_task] running split_keyboard_task");

    loop {
        // wait till new key_report is received from key_provision
        let message: [u8; 6] = message_to_peri.changed().await;

        // write to characteristic
        client
            .write_characteristic_without_response(&characteristic, &message)
            .await
            .expect("[ble_central] error sending message to peri");
    }
}
