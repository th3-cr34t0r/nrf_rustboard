use core::ops::Range;
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

fn flash_range<S: NorFlash>() -> Range<u32> {
    0..2 * S::ERASE_SIZE as u32
}

pub async fn store_bonding_info<S: NorFlash>(
    storage: &mut S,
    info: &BondInformation,
) -> Result<(), sequential_storage::Error<S::Error>> {
    let start_addr = 0xA0000 as u32;
    let storage_range = start_addr..(start_addr + 8 * S::ERASE_SIZE as u32);

    sequential_storage::erase_all(storage, storage_range.clone()).await?;

    let mut buffer = [0; 32];
    let key = StoredAddr(info.identity.bd_addr);
    let value = StoredBondInformation {
        ltk: info.ltk,
        security_level: info.security_level,
    };

    sequential_storage::map::store_item(
        storage,
        flash_range::<S>(),
        &mut NoCache::new(),
        &mut buffer,
        &key,
        &value,
    )
    .await?;

    Ok(())
}

pub async fn load_bonding_info<S: NorFlash>(storage: &mut S) -> Option<BondInformation> {
    let mut buffer = [0; 32];
    let mut cache = NoCache::new();

    let mut iter =
        fetch_all_items::<StoredAddr, _, _>(storage, flash_range::<S>(), &mut cache, &mut buffer)
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
