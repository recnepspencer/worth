use super::durable_frame::{damaged_field, read_u16, read_u64};
use super::physical_fields::{scope, shape};
use crate::integrity_observation::{
    record_walk::damage, sha256::sha256, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflinePhysicalFormatField as Field,
    OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};
use worth_foundational::PhysicalByteRange;
use worth_store_physical_format::integrity_declarations::families::{
    WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES, WAL_FRAME_V1_VERSION,
};

pub(crate) struct WalFrameObservation {
    pub(crate) offset: u64,
    pub(crate) length: u64,
    pub(crate) outcome: Outcome,
    pub(crate) lsn: Option<(u64, u64)>,
}

pub(crate) fn read_wal_segment(
    bytes: &[u8],
    segment: u64,
    generation: u64,
    maximum_frames: u64,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Vec<WalFrameObservation> {
    let mut observations = Vec::new();
    if bytes.is_empty() {
        return vec![WalFrameObservation {
            offset: 0,
            length: 0,
            lsn: None,
            outcome: damage(
                Cause::Truncation,
                Some((0, WAL_FRAME_V1_HEADER_BYTES as u64)),
                Blast::Artifact,
            ),
        }];
    }
    let mut offset = 0;
    let mut next_lsn = None;
    while offset < bytes.len() {
        if observations.len() as u64 >= maximum_frames {
            observations.push(WalFrameObservation {
                lsn: None,
                offset: offset as u64,
                length: 0,
                outcome: Outcome::Indeterminate(
                    crate::OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
                ),
            });
            break;
        }
        let remaining = &bytes[offset..];
        match read_frame(remaining, segment, generation, next_lsn, counters) {
            Ok((length, end)) => {
                observations.push(WalFrameObservation {
                    lsn: Some((read_u64(remaining, 28), end)),
                    offset: offset as u64,
                    length: length as u64,
                    outcome: Outcome::Intact,
                });
                offset += length;
                next_lsn = Some(end);
            }
            Err(outcome) => {
                observations.push(WalFrameObservation {
                    lsn: None,
                    offset: offset as u64,
                    length: remaining.len() as u64,
                    outcome: super::super::record_walk::shift_outcome(outcome, offset as u64),
                });
                break;
            }
        }
    }
    observations
}

fn read_frame(
    bytes: &[u8],
    segment: u64,
    generation: u64,
    next_lsn: Option<u64>,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<(usize, u64), Outcome> {
    if bytes.len() < WAL_FRAME_V1_HEADER_BYTES {
        return Err(damage(
            Cause::Truncation,
            Some((
                bytes.len() as u64,
                (WAL_FRAME_V1_HEADER_BYTES - bytes.len()) as u64,
            )),
            Blast::Frame,
        ));
    }
    if &bytes[..8] != b"WORTHWAL" {
        return Err(damaged_field(Cause::Framing, 0, 8, Field::Magic));
    }
    let version = read_u16(bytes, 8);
    if version != WAL_FRAME_V1_VERSION {
        return Err(Outcome::Unsupported(
            OfflineUnsupportedPhysicalVersion::new(
                OfflineUnsupportedVersionAxis::WalFrame,
                u64::from(version),
                "1",
                PhysicalByteRange::new(8, 2).unwrap(),
            ),
        ));
    }
    shape(
        usize::from(read_u16(bytes, 10)) == WAL_FRAME_V1_HEADER_BYTES,
        10,
        2,
    )?;
    let payload = read_u64(bytes, 44);
    let length = payload
        .checked_add((WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64)
        .and_then(|length| usize::try_from(length).ok())
        .ok_or_else(|| damaged_field(Cause::Framing, 44, 8, Field::PayloadLength))?;
    shape(payload != 0, 44, 8)?;
    if length > bytes.len() {
        return Err(damage(
            Cause::Truncation,
            Some((bytes.len() as u64, (length - bytes.len()) as u64)),
            Blast::Frame,
        ));
    }
    let footer = length - WAL_FRAME_V1_FOOTER_BYTES;
    counters.checksum_calculations += 1;
    if sha256(&bytes[WAL_FRAME_V1_HEADER_BYTES..footer]) != bytes[84..116] {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((WAL_FRAME_V1_HEADER_BYTES as u64, payload)),
            Blast::Frame,
        ));
    }
    counters.checksum_calculations += 1;
    if sha256(&bytes[..footer]) != bytes[footer..length] {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((0, footer as u64)),
            Blast::Frame,
        ));
    }
    scope(
        read_u64(bytes, 12) == segment && read_u64(bytes, 20) == generation,
        12,
        16,
        Field::IdentityField,
    )?;
    let start = read_u64(bytes, 28);
    let end = read_u64(bytes, 36);
    scope(
        start < end && next_lsn.is_none_or(|next| start == next),
        28,
        16,
        Field::WalLsn,
    )?;
    Ok((length, end))
}
