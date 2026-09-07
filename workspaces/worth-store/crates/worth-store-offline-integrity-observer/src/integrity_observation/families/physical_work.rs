use super::{
    durable_frame::{damaged_field, read_u64},
    physical_fields::{scope, shape},
};
use crate::integrity_observation::{
    record_walk::damage, sha256::sha256, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflinePhysicalFormatField as Field,
    OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};
use worth_foundational::PhysicalByteRange;
use worth_store_physical_format::integrity_declarations::families::{
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};

pub(crate) fn read_physical_work(
    bytes: &[u8],
    store: [u8; 16],
    identity: (u64, u64, u64),
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<(), Outcome> {
    if bytes.len() < PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES {
        return Err(damage(
            Cause::Truncation,
            Some((
                bytes.len() as u64,
                (PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES - bytes.len()) as u64,
            )),
            Blast::Artifact,
        ));
    }
    shape(
        bytes.len() == PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
        0,
        bytes.len(),
    )?;
    if &bytes[..8] != b"WPEFFECT" {
        return Err(damaged_field(Cause::Framing, 0, 8, Field::Magic));
    }
    if bytes[8] != PHYSICAL_WORK_OBLIGATION_V6_VERSION {
        return Err(Outcome::Unsupported(
            OfflineUnsupportedPhysicalVersion::new(
                OfflineUnsupportedVersionAxis::PhysicalWork,
                u64::from(bytes[8]),
                "6",
                PhysicalByteRange::new(8, 1).unwrap(),
            ),
        ));
    }
    shape(
        bytes[10..16] == [0; 6] && bytes[107..112] == [0; 5],
        10,
        102,
    )?;
    counters.checksum_calculations += 1;
    if sha256(&bytes[..128]) != bytes[128..160] {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((0, 128)),
            Blast::Artifact,
        ));
    }
    scope(bytes[16..32] == store, 16, 16, Field::StoreIdentity)?;
    scope(
        (
            read_u64(bytes, 32),
            read_u64(bytes, 40),
            read_u64(bytes, 48),
        ) == identity,
        32,
        24,
        Field::IdentityField,
    )?;
    shape(matches!(bytes[9], 1..=9) && bytes[105] <= 1, 9, 97)?;
    let offset = read_u64(bytes, 56);
    let count = read_u64(bytes, 64);
    let first = read_u64(bytes, 112);
    let second = read_u64(bytes, 120);
    let digest = bytes[105] == 1;
    let empty = offset == 0 && count == 0;
    let interval = count > 0 && offset.checked_add(count).is_some();
    let artifact = match bytes[106] {
        1 | 12 | 13 => first == 0 && second == 0,
        2 | 3 | 10 | 14 | 15 => second == 0,
        4..=9 | 11 => true,
        _ => false,
    };
    let valid = match bytes[104] {
        1 => digest && interval && artifact,
        2 | 3 => empty && !digest && artifact,
        4 => empty && !digest && bytes[106] == 2 && second == 0,
        5 => empty && !digest && bytes[106] == 0 && first == 0 && second == 0,
        6 => digest && interval && bytes[106] == 0 && first > 0 && second > 0,
        7 => {
            first > 0
                && second == 0
                && match bytes[106] {
                    1 => digest && count > 0 && offset == 0,
                    2 => digest && interval,
                    3..=6 => !digest && empty,
                    _ => false,
                }
        }
        8 => !digest && empty && bytes[106] == 0 && first > 0 && second > 0,
        _ => false,
    };
    shape(valid, 56, 72)
}
