use sha2::{Digest, Sha256};

use super::configuration::BoundedResidencyConfiguration;

#[derive(Debug, Clone, Copy)]
pub(super) struct ExpectedDurableTruth {
    seed: u64,
    records: usize,
    payload_bytes: u64,
    digest: [u8; 32],
}

impl ExpectedDurableTruth {
    pub(super) const fn seed(self) -> u64 {
        self.seed
    }
    pub(super) const fn records(self) -> usize {
        self.records
    }
    pub(super) const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }
    pub(super) const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

pub(super) fn derive(
    configuration: BoundedResidencyConfiguration,
) -> Result<ExpectedDurableTruth, String> {
    let mut digest = Sha256::new();
    let mut payload_bytes = 0_u64;
    let mut buffer = [0_u8; 8_192];
    for ordinal in 0..configuration.record_count() {
        let record_bytes = configuration
            .record_bytes(ordinal)
            .ok_or_else(|| "expected record ordinal is outside configuration".to_owned())?;
        digest.update((record_bytes as u64).to_le_bytes());
        let mut offset = 0_usize;
        while offset < record_bytes {
            let width = (record_bytes - offset).min(buffer.len());
            for (relative, byte) in buffer[..width].iter_mut().enumerate() {
                *byte = expected_byte(configuration.seed(), ordinal as u64, offset + relative);
            }
            digest.update(&buffer[..width]);
            offset += width;
        }
        payload_bytes = payload_bytes
            .checked_add(record_bytes as u64)
            .ok_or_else(|| "expected payload bytes overflowed".to_owned())?;
    }
    Ok(ExpectedDurableTruth {
        seed: configuration.seed(),
        records: configuration.record_count(),
        payload_bytes,
        digest: digest.finalize().into(),
    })
}

pub(super) fn record_digest(
    configuration: BoundedResidencyConfiguration,
    ordinal: usize,
) -> Result<[u8; 32], String> {
    let record_bytes = configuration
        .record_bytes(ordinal)
        .ok_or_else(|| format!("expected record ordinal {ordinal} is outside configuration"))?;
    let mut digest = Sha256::new();
    for offset in 0..record_bytes {
        digest.update([expected_byte(configuration.seed(), ordinal as u64, offset)]);
    }
    Ok(digest.finalize().into())
}

fn payload_byte(seed: u64, ordinal: u64, offset: u64) -> u8 {
    let mixed = seed
        ^ ordinal.wrapping_add(1).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ offset.wrapping_add(1).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    avalanche(mixed) as u8
}

fn expected_byte(seed: u64, ordinal: u64, offset: usize) -> u8 {
    if offset < 8 {
        ordinal.to_le_bytes()[offset]
    } else {
        payload_byte(seed, ordinal, offset as u64)
    }
}

const fn avalanche(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
