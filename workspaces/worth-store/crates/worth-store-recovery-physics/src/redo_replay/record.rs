use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use worth_store_physical_format::{
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryProjection,
    PhysicalRecoveryProjectionDecodeLimits, RecordArtifactFile,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::PhysicalRedoPlanningDenial;

mod target;
mod target_decode;
use target_decode::decode_targets;
#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysicalRedoTargetIdentity {
    InlinePage {
        segment: u64,
        page: u64,
        generation: u64,
    },
    ExtentChunk {
        extent: u64,
        generation: u64,
        chunk: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRedoExtentCoordinate {
    allocation_epoch: [u8; 16],
    record_ordinal: u64,
    logical_bytes: u64,
    logical_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoTarget {
    identity: PhysicalRedoTargetIdentity,
    extent_coordinate: Option<PhysicalRedoExtentCoordinate>,
    artifact: RecordArtifactFile,
    artifact_offset: u64,
    artifact_length: u32,
    resulting_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoRecord {
    ordinal: u32,
    lsn: LogSequenceNumber,
    targets: Box<[PhysicalRedoTarget]>,
    bytes: Box<[u8]>,
}

pub fn decode_physical_redo_records(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
) -> Result<Box<[PhysicalRedoRecord]>, PhysicalRedoPlanningDenial> {
    decode_member(
        bytes,
        expected_range,
        maximum_targets,
        None,
        default_projection_limits(maximum_targets),
    )
    .map(|(records, _)| records)
}

pub(super) fn decode_physical_redo_member(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    distinct: Option<(&mut BTreeSet<PhysicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
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
    )
}

pub(super) fn decode_physical_redo_records_with_distinct(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    distinct: &mut BTreeSet<PhysicalRedoTargetIdentity>,
    maximum_distinct: u64,
) -> Result<Box<[PhysicalRedoRecord]>, PhysicalRedoPlanningDenial> {
    decode_member(
        bytes,
        expected_range,
        maximum_targets,
        Some((distinct, maximum_distinct)),
        default_projection_limits(maximum_targets),
    )
    .map(|(records, _)| records)
}

fn decode_member(
    bytes: &[u8],
    expected_range: WalLsnRange,
    maximum_targets: u64,
    mut distinct: Option<(&mut BTreeSet<PhysicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
) -> Result<
    (
        Box<[PhysicalRedoRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    PhysicalRedoPlanningDenial,
> {
    let mut cursor = Cursor::new(bytes);
    if cursor.field()? != REDO_DOMAIN {
        return Err(PhysicalRedoPlanningDenial::WrongDomain);
    }
    let count = cursor.u64()?;
    if count == 0 || count != expected_range.end_exclusive().get() - expected_range.start().get() {
        return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
    }
    if count > maximum_targets {
        return Err(PhysicalRedoPlanningDenial::TargetLimit);
    }
    let capacity =
        usize::try_from(count).map_err(|_| PhysicalRedoPlanningDenial::RecordCountLimit)?;
    let mut records = Vec::with_capacity(capacity);
    let mut target_count = 0_u64;
    for expected_ordinal in 0..count {
        let ordinal = cursor.u32()?;
        let lsn = cursor.u64()?;
        if u64::from(ordinal) != expected_ordinal
            || lsn != expected_range.start().get() + expected_ordinal
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecordOrder);
        }
        let targets = decode_targets(
            &mut cursor,
            &mut target_count,
            maximum_targets,
            &mut distinct,
        )?;
        let record_bytes = cursor.field()?;
        if record_bytes.is_empty() {
            return Err(PhysicalRedoPlanningDenial::MalformedMember);
        }
        records.push(PhysicalRedoRecord {
            ordinal,
            lsn: LogSequenceNumber::new(lsn),
            targets,
            bytes: record_bytes.into(),
        });
    }
    let projection =
        PersistedPhysicalRecoveryProjection::decode_frames(cursor.field()?, projection_limits)
            .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    cursor.require_end()?;
    validate_projection(&records, &projection)?;
    Ok((records.into_boxed_slice(), projection))
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

fn validate_projection(
    records: &[PhysicalRedoRecord],
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let frames = projection
        .frames()
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let targets = records
        .iter()
        .flat_map(|record| record.targets())
        .collect::<Vec<_>>();
    for target in &targets {
        let matches = frames
            .iter()
            .filter(|frame| materialization_matches(target, frame))
            .count();
        if matches != 1 {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    if frames.iter().any(|frame| {
        !targets
            .iter()
            .any(|target| materialization_matches(target, frame))
    }) {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

fn materialization_matches(
    target: &PhysicalRedoTarget,
    frame: &worth_store_physical_format::PersistedPhysicalRecoveryFrame,
) -> bool {
    let identity_matches = match (target.identity(), frame.subject()) {
        (
            PhysicalRedoTargetIdentity::InlinePage {
                segment,
                page,
                generation,
            },
            PersistedPhysicalDataFrameSubject::InlinePage(subject),
        ) => {
            (segment, page, generation)
                == (
                    subject.segment_id().get(),
                    subject.page_id().get(),
                    subject.generation().get(),
                )
        }
        (
            PhysicalRedoTargetIdentity::ExtentChunk {
                extent,
                generation,
                chunk,
            },
            PersistedPhysicalDataFrameSubject::ExtentChunk(subject),
        ) => {
            (extent, generation, chunk)
                == (
                    subject.extent_cell().extent_id().get(),
                    subject.extent_cell().generation().get(),
                    subject.ordinal(),
                )
        }
        _ => false,
    };
    let coordinate = frame.coordinate();
    identity_matches
        && coordinate.artifact() == target.artifact()
        && coordinate.offset() == target.artifact_offset()
        && coordinate.length() == target.artifact_length()
        && Sha256::digest(frame.bytes()).as_slice() == target.resulting_digest()
}

struct Cursor<'a> {
    remaining: &'a [u8],
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }
    fn byte(&mut self) -> Result<u8, PhysicalRedoPlanningDenial> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, PhysicalRedoPlanningDenial> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, PhysicalRedoPlanningDenial> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], PhysicalRedoPlanningDenial> {
        self.take(N)?
            .try_into()
            .map_err(|_| PhysicalRedoPlanningDenial::MalformedMember)
    }
    fn field(&mut self) -> Result<&'a [u8], PhysicalRedoPlanningDenial> {
        let len = usize::try_from(self.u64()?)
            .map_err(|_| PhysicalRedoPlanningDenial::MalformedMember)?;
        self.take(len)
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8], PhysicalRedoPlanningDenial> {
        let (head, tail) = self
            .remaining
            .split_at_checked(len)
            .ok_or(PhysicalRedoPlanningDenial::MalformedMember)?;
        self.remaining = tail;
        Ok(head)
    }
    fn require_end(self) -> Result<(), PhysicalRedoPlanningDenial> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(PhysicalRedoPlanningDenial::MalformedMember)
        }
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
