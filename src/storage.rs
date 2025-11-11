use core::ops::Range;
use defmt::info;
use embedded_storage_async::nor_flash::NorFlash;
use sequential_storage::cache::NoCache;
use sequential_storage::map::{Key, SerializationError, Value, fetch_all_items};
use trouble_host::prelude::{BdAddr, SecurityLevel};
use trouble_host::{BondInformation, Identity, LongTermKey};

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredAddr(BdAddr);

impl Key for StoredAddr {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        if buffer.len() < 6 {
            Err(SerializationError::BufferTooSmall)
        } else {
            buffer[0..6].copy_from_slice(self.0.raw());
            Ok(6)
        }
    }
    fn deserialize_from(buffer: &[u8]) -> Result<(Self, usize), SerializationError> {
        if buffer.len() < 6 {
            Err(SerializationError::BufferTooSmall)
        } else {
            Ok((
                StoredAddr(BdAddr::new(
                    buffer[0..6]
                        .try_into()
                        .expect("[deserialize_from] error deserializing buffer"),
                )),
                6,
            ))
        }
    }
}

struct StoredBondInformation {
    ltk: LongTermKey,
    security_level: SecurityLevel,
}

impl<'a> Value<'a> for StoredBondInformation {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        if buffer.len() < 17 {
            return Err(SerializationError::BufferTooSmall);
        }
        buffer[0..16].copy_from_slice(self.ltk.to_le_bytes().as_slice());

        buffer[16] = match self.security_level {
            SecurityLevel::NoEncryption => 0,
            SecurityLevel::Encrypted => 1,
            SecurityLevel::EncryptedAuthenticated => 2,
        };
        Ok(17)
    }

    fn deserialize_from(buffer: &'a [u8]) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        if buffer.len() < 17 {
            Err(SerializationError::BufferTooSmall)
        } else {
            let ltk = LongTermKey::from_le_bytes(buffer[0..16].try_into().unwrap());
            let security_level = match buffer[16] {
                0 => SecurityLevel::NoEncryption,
                1 => SecurityLevel::Encrypted,
                2 => SecurityLevel::EncryptedAuthenticated,
                _ => return Err(SerializationError::InvalidData),
            };
            Ok(StoredBondInformation {
                ltk,
                security_level,
            })
        }
    }
}

pub async fn store_bonding_info<S: NorFlash>(
    storage: &mut S,
    bond_informaton: &BondInformation,
) -> Result<(), sequential_storage::Error<S::Error>> {
    // start address
    let start_addr = 0xA0000 as u32;
    let storage_range = start_addr..(start_addr + 8 * S::ERASE_SIZE as u32);

    info!(
        "[store_bonding_info] storage: {}kb, start_address: {}, storage_range: {}",
        storage.capacity(),
        start_addr,
        storage_range,
    );

    sequential_storage::erase_all(storage, storage_range.clone()).await?;

    info!("[store_bonding_info] erased");

    let mut buffer = [0; 32];
    let key = StoredAddr(bond_informaton.identity.bd_addr);
    let value = StoredBondInformation {
        ltk: bond_informaton.ltk,
        security_level: bond_informaton.security_level,
    };

    sequential_storage::map::store_item(
        storage,
        storage_range,
        &mut NoCache::new(),
        &mut buffer,
        &key,
        &value,
    )
    .await?;

    info!("[store_bonding_info] stored key-value");

    Ok(())
}

pub async fn load_bonding_info<S: NorFlash>(storage: &mut S) -> Option<BondInformation> {
    let start_addr = 0xA0000 as u32;
    let storage_range = start_addr..(start_addr + 8 * S::ERASE_SIZE as u32);

    let mut buffer = [0; 32];
    let mut cache = NoCache::new();

    let mut iter =
        fetch_all_items::<StoredAddr, _, _>(storage, storage_range, &mut cache, &mut buffer)
            .await
            .ok()?;

    while let Some((key, value)) = iter.next::<StoredBondInformation>(&mut buffer).await.ok()? {
        return Some(BondInformation {
            ltk: value.ltk,
            identity: Identity {
                bd_addr: key.0,
                irk: None,
            },
            is_bonded: true,
            security_level: value.security_level,
        });
    }
    None
}
