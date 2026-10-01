use std::collections::BTreeSet;

use worth_store_physical_format::{
    decode_canonical_redo_v3, CanonicalRedoWireDenial, PersistedPhysicalRecoveryProjection,
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::PhysicalRedoPlanningDenial;

pub use worth_store_physical_format::{
    CanonicalRedoExtentCoordinate as PhysicalRedoExtentCoordinate,
    CanonicalRedoTarget as PhysicalRedoTarget,
    CanonicalRedoTargetIdentity as PhysicalRedoTargetIdentity,
};

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;

#[cfg(test)]
const REDO_DOMAIN: &[u8] = worth_store_physical_format::CANONICAL_REDO_V3_DOMAIN;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoRecord {
    ordinal: u32,
    lsn: LogSequenceNumber,
    targets: Box<[PhysicalRedoTarget]>,
    bytes: Box<[u8]>,
}

impl PhysicalRedoRecord {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let targets = u64::try_from(self.targets.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<PhysicalRedoTarget>()).ok()?)?;
        targets.checked_add(u64::try_from(self.bytes.len()).ok()?)
    }
}

#[cfg(test)]
mod retained_storage_tests {
    use super::*;

    #[test]
    fn record_counts_owned_redo_bytes() {
        let record = PhysicalRedoRecord {
            ordinal: 0,
            lsn: LogSequenceNumber::new(1),
            targets: Box::new([]),
            bytes: vec![1, 2, 3, 4, 5].into_boxed_slice(),
        };
        assert_eq!(record.owned_heap_bytes(), Some(5));
    }
}

pub fn decode_physical_redo_records(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<Box<[PhysicalRedoRecord]>, PhysicalRedoPlanningDenial> {
    decode_member(
        bytes,
        expected_range,
        maximum_targets,
        None,
        default_projection_limits(maximum_targets),
        format,
    )
    .map(|(records, _)| records)
}

pub(super) fn decode_physical_redo_member(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    distinct: Option<(&mut BTreeSet<PhysicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
    format: PhysicalRecordFormatDeclaration,
) -> Result<
    (
        Box<[PhysicalRedoRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    PhysicalRedoPlanningDenial,
> {
    decode_member(
        bytes,
        expected_range,
        maximum_targets,
        distinct,
        projection_limits,
        format,
    )
}

pub(super) fn decode_physical_redo_records_with_distinct(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    distinct: &mut BTreeSet<PhysicalRedoTargetIdentity>,
    maximum_distinct: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<Box<[PhysicalRedoRecord]>, PhysicalRedoPlanningDenial> {
    decode_member(
        bytes,
        expected_range,
        maximum_targets,
        Some((distinct, maximum_distinct)),
        default_projection_limits(maximum_targets),
        format,
    )
    .map(|(records, _)| records)
}

fn decode_member(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    distinct: Option<(&mut BTreeSet<PhysicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
    format: PhysicalRecordFormatDeclaration,
) -> Result<
    (
        Box<[PhysicalRedoRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    PhysicalRedoPlanningDenial,
> {
    let (wire_records, projection) = decode_canonical_redo_v3(
        bytes,
        expected_range.start().get(),
        expected_range.end_exclusive().get(),
        maximum_targets,
        distinct,
        projection_limits,
        format,
    )
    .map_err(map_wire_denial)?;
    let records = wire_records
        .into_vec()
        .into_iter()
        .map(|wire| {
            let (ordinal, lsn, targets, bytes) = wire.into_parts();
            PhysicalRedoRecord {
                ordinal,
                lsn: LogSequenceNumber::new(lsn),
                targets,
                bytes,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok((records, projection))
}

const fn default_projection_limits(maximum: u64) -> PhysicalRecoveryProjectionDecodeLimits {
    PhysicalRecoveryProjectionDecodeLimits {
        frames: maximum,
        record_identities: maximum,
        placements: maximum,
        segment_updates: maximum,
        manifests: maximum,
        total_entries: maximum.saturating_mul(3),
        inline_allocations: maximum,
    }
}

const fn map_wire_denial(denial: CanonicalRedoWireDenial) -> PhysicalRedoPlanningDenial {
    match denial {
        CanonicalRedoWireDenial::MalformedMember => PhysicalRedoPlanningDenial::MalformedMember,
        CanonicalRedoWireDenial::WrongDomain => PhysicalRedoPlanningDenial::WrongDomain,
        CanonicalRedoWireDenial::RecordCountLimit => PhysicalRedoPlanningDenial::RecordCountLimit,
        CanonicalRedoWireDenial::TargetLimit => PhysicalRedoPlanningDenial::TargetLimit,
        CanonicalRedoWireDenial::DistinctTargetLimit => {
            PhysicalRedoPlanningDenial::DistinctTargetLimit
        }
        CanonicalRedoWireDenial::InvalidRecordOrder => {
            PhysicalRedoPlanningDenial::InvalidRecordOrder
        }
        CanonicalRedoWireDenial::NonCanonicalTargetOrder => {
            PhysicalRedoPlanningDenial::NonCanonicalTargetOrder
        }
        CanonicalRedoWireDenial::LsnRangeMismatch => PhysicalRedoPlanningDenial::LsnRangeMismatch,
        CanonicalRedoWireDenial::InvalidTarget => PhysicalRedoPlanningDenial::InvalidTarget,
        CanonicalRedoWireDenial::InvalidRecoveryProjection => {
            PhysicalRedoPlanningDenial::InvalidRecoveryProjection
        }
        CanonicalRedoWireDenial::UnsupportedRecoveryProjectionVersion(version) => {
            PhysicalRedoPlanningDenial::UnsupportedRecoveryProjectionVersion(version)
        }
        CanonicalRedoWireDenial::ProjectionEntryLimit => {
            PhysicalRedoPlanningDenial::InvalidRecoveryProjection
        }
        CanonicalRedoWireDenial::CounterOverflow => PhysicalRedoPlanningDenial::CounterOverflow,
    }
}

impl PhysicalRedoRecord {
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
    pub const fn lsn(&self) -> LogSequenceNumber {
        self.lsn
    }
    pub fn targets(&self) -> &[PhysicalRedoTarget] {
        &self.targets
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
