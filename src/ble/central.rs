// ble central
//

use embassy_executor::Spawner;
use embedded_storage_async::nor_flash::NorFlash;
use nrf_mpsl::MultiprotocolServiceLayer;
use nrf_sdc::SoftdeviceController;
use rand::{CryptoRng, RngCore};
use trouble_host::Address;

/// run ble
pub async fn ble_central_run<RNG, S>(
    sdc: SoftdeviceController<'static>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    mut storage: &mut S,
    rng: &mut RNG,
    spawner: Spawner,
) where
    RNG: RngCore + CryptoRng,
    S: NorFlash,
{
    // let addr_0 = FICR.deviceaddr(0).read();
    // let addr_1 = FICR.deviceaddr(1).read();

    // let address = Address::random([]);
}
