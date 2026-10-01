use super::super::{
    durable_frame::{damaged_field, read_u32},
    physical_fields::shape,
};
use crate::integrity_observation::{
    crc32c::crc32c, record_walk::damage, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflinePhysicalFormatField as Field,
    OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};
use worth_foundational::{PhysicalArtifactFamily as Family, PhysicalByteRange};

pub(super) fn read_record<'a>(
    bytes: &'a [u8],
    kind: u8,
    expected_schema: Option<u8>,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<(&'a [u8], usize), Outcome> {
    if bytes.len() < 16 {
        return Err(damage(
            Cause::Truncation,
            Some((bytes.len() as u64, (16 - bytes.len()) as u64)),
            Blast::Frame,
        ));
    }
    if &bytes[..8] != b"WCP7REC\0" {
        return Err(damaged_field(Cause::Framing, 0, 8, Field::Magic));
    }
    if !matches!(bytes[8], 1 | 2 | 3) {
        return Err(Outcome::Unsupported(
            OfflineUnsupportedPhysicalVersion::new(
                OfflineUnsupportedVersionAxis::CheckpointRecord,
                u64::from(bytes[8]),
                "1|2|3",
                PhysicalByteRange::new(8, 1).unwrap(),
            ),
        ));
    }
    if expected_schema.is_some_and(|schema| schema != bytes[8]) {
        return Err(damaged_field(
            Cause::ScopeMismatch,
            8,
            1,
            Field::EnvelopeSchema,
        ));
    }
    if bytes[9] != kind {
        return Err(damaged_field(Cause::ScopeMismatch, 9, 1, Field::FamilyKind));
    }
    shape(bytes[10..12] == [0; 2], 10, 2)?;
    let payload = read_u32(bytes, 12) as usize;
    let valid = match kind {
        1 => payload == 144,
        2 => payload == 48,
        3 => payload == 16,
        4 => (1..=4096).contains(&payload),
        5 => payload == if bytes[8] == 3 { 184 } else { 136 },
        6 | 7 => bytes[8] == 3 && (1..=65_516).contains(&payload),
        _ => false,
    };
    shape(valid, 12, 4)?;
    let length = 20 + payload;
    if bytes.len() < length {
        return Err(damage(
            Cause::Truncation,
            Some((bytes.len() as u64, (length - bytes.len()) as u64)),
            Blast::Frame,
        ));
    }
    counters.checksum_calculations += 1;
    if crc32c(&[&bytes[..length - 4]]) != read_u32(bytes, length - 4) {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((0, (length - 4) as u64)),
            Blast::Frame,
        ));
    }
    Ok((&bytes[16..length - 4], length))
}

pub(super) fn family(kind: u8) -> Family {
    match kind {
        1 => Family::CheckpointStreamHeader,
        2 => Family::CheckpointDirtyBasis,
        3 => Family::CheckpointBindingCompaction,
        4 => Family::CheckpointBinding,
        5 => Family::CheckpointFooter,
        6 => Family::CheckpointTierCertificate,
        _ => Family::CheckpointReleaseCertificate,
    }
}
