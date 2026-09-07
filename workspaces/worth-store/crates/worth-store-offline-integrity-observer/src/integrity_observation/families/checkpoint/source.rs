use super::super::{
    durable_frame::{read_u32, read_u64},
    physical_fields::{scope, shape},
};
use crate::integrity_observation::{
    sha256::Sha256, OfflineIntegrityOutcome as Outcome, OfflinePhysicalFormatField as Field,
};

pub(super) struct Source {
    pub(super) identity: [u8; 24],
    pub(super) sequence: u64,
    pub(super) wal_end: u64,
}

pub(super) fn read_source(
    payload: &[u8],
    store: [u8; 16],
    sequence: Option<u64>,
) -> Result<Source, Outcome> {
    scope(payload[..16] == store, 16, 16, Field::StoreIdentity)?;
    let found = read_u64(payload, 16);
    scope(
        found != 0 && sequence.is_none_or(|sequence| found == sequence),
        32,
        8,
        Field::IdentityField,
    )?;
    scope(
        read_u64(payload, 24) < read_u64(payload, 32),
        40,
        16,
        Field::WalLsn,
    )?;
    shape(payload[64] == 1 && payload[66..72] == [0; 6], 80, 8)?;
    match payload[65] {
        0 => shape(payload[72..144] == [0; 72], 88, 72)?,
        1 => {
            shape(
                payload[72..104] != [0; 32] && read_u64(payload, 104) != 0,
                88,
                40,
            )?;
            let mut digest = Sha256::new();
            digest.update(b"worth.store.checkpoint-security-binding.v1");
            digest.update(&payload[..56]);
            digest.update(&payload[72..112]);
            scope(
                digest.finish() == payload[112..144],
                128,
                32,
                Field::Checksum,
            )?;
        }
        _ => shape(false, 81, 1)?,
    }
    Ok(Source {
        identity: payload[..24].try_into().unwrap(),
        sequence: found,
        wal_end: read_u64(payload, 32),
    })
}

pub(super) fn read_dirty(payload: &[u8]) -> Result<(), Outcome> {
    shape(payload[1..8] == [0; 7] && payload[36..40] == [0; 4], 17, 39)?;
    let first = read_u64(payload, 8);
    let second = read_u64(payload, 16);
    let valid = match payload[0] {
        1 | 12 | 13 => first == 0 && second == 0,
        2 | 3 | 10 | 14 | 15 => second == 0,
        4..=9 | 11 => true,
        _ => false,
    };
    scope(valid, 16, 24, Field::IdentityField)?;
    let length = u64::from(read_u32(payload, 32));
    shape(
        length > 0 && read_u64(payload, 24).checked_add(length).is_some(),
        40,
        12,
    )
}
