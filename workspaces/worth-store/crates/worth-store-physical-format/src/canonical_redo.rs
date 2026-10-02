//! Bounded, format-owned decoding of canonical physical redo v3.
//!
//! This admits wire syntax and its exact frame projection, but does not decide
//! recovery fate or authorize replay. Those remain C8 responsibilities.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::recovery_projection::decode_storage::{
    canonical_denial, copy_box, reserve_vec, UnrestrictedDecodeStorage,
};
use crate::{
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryProjection,
    PhysicalRecordFormatDeclaration, PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage,
    PhysicalRecoveryProjectionDecodeLimits, RecordArtifactFile,
};

mod target;
mod target_decode;
use target_decode::decode_targets;

#[cfg(test)]
#[path = "canonical_redo/tests.rs"]
mod tests;

pub const CANONICAL_REDO_V3_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalRedoTargetIdentity {
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
pub struct CanonicalRedoExtentCoordinate {
    allocation_epoch: [u8; 16],
    record_ordinal: u64,
    logical_bytes: u64,
    logical_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRedoTarget {
    identity: CanonicalRedoTargetIdentity,
    extent_coordinate: Option<CanonicalRedoExtentCoordinate>,
    artifact: RecordArtifactFile,
    artifact_offset: u64,
    artifact_length: u32,
    resulting_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRedoWireRecord {
    ordinal: u32,
    lsn: u64,
    targets: Box<[CanonicalRedoTarget]>,
    bytes: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalRedoWireDenial {
    MalformedMember,
    WrongDomain,
    RecordCountLimit,
    TargetLimit,
    DistinctTargetLimit,
    InvalidRecordOrder,
    NonCanonicalTargetOrder,
    LsnRangeMismatch,
    InvalidTarget,
    InvalidRecoveryProjection,
    UnsupportedRecoveryProjectionVersion(u16),
    ProjectionEntryLimit,
    CounterOverflow,
}

/// Decodes one canonical v3 member. Numeric half-open LSNs keep the format
/// independent of WAL ownership; callers supply the already-admitted range.
pub fn decode_canonical_redo_v3(
    bytes: &[u8],
    expected_start_lsn: u64,
    expected_end_lsn_exclusive: u64,
    maximum_targets: u64,
    distinct: Option<(&mut BTreeSet<CanonicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
    format: PhysicalRecordFormatDeclaration,
) -> Result<
    (
        Box<[CanonicalRedoWireRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    CanonicalRedoWireDenial,
> {
    decode_in_storage(
        bytes,
        expected_start_lsn,
        expected_end_lsn_exclusive,
        maximum_targets,
        distinct,
        projection_limits,
        format,
        &mut UnrestrictedDecodeStorage,
    )
    .map_err(canonical_denial)
}

/// Decode with caller-carried admission for every owned allocation and bounded
/// verification scratch. Cross-member distinct-set policy remains its caller's
/// responsibility; this path performs the same per-member format validation.
pub fn decode_canonical_redo_v3_with_storage<S: PhysicalRecoveryDecodeStorage>(
    bytes: &[u8],
    expected_start_lsn: u64,
    expected_end_lsn_exclusive: u64,
    maximum_targets: u64,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<
    (
        Box<[CanonicalRedoWireRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    PhysicalRecoveryDecodeFailure<S::Denial>,
> {
    decode_in_storage(
        bytes,
        expected_start_lsn,
        expected_end_lsn_exclusive,
        maximum_targets,
        None,
        projection_limits,
        format,
        storage,
    )
}

fn decode_in_storage<S: PhysicalRecoveryDecodeStorage>(
    bytes: &[u8],
    expected_start_lsn: u64,
    expected_end_lsn_exclusive: u64,
    maximum_targets: u64,
    mut distinct: Option<(&mut BTreeSet<CanonicalRedoTargetIdentity>, u64)>,
    projection_limits: PhysicalRecoveryProjectionDecodeLimits,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<
    (
        Box<[CanonicalRedoWireRecord]>,
        PersistedPhysicalRecoveryProjection,
    ),
    PhysicalRecoveryDecodeFailure<S::Denial>,
> {
    let mut cursor = Cursor::new(bytes);
    if cursor.field()? != CANONICAL_REDO_V3_DOMAIN {
        return Err(CanonicalRedoWireDenial::WrongDomain.into());
    }
    let count = cursor.u64()?;
    if count == 0 || count != expected_end_lsn_exclusive.saturating_sub(expected_start_lsn) {
        return Err(CanonicalRedoWireDenial::LsnRangeMismatch.into());
    }
    if count > maximum_targets {
        return Err(CanonicalRedoWireDenial::TargetLimit.into());
    }
    cursor.require_count_backing(count, 4 + 8 + 8 + 8)?;
    let capacity = usize::try_from(count).map_err(|_| CanonicalRedoWireDenial::RecordCountLimit)?;
    let mut records = reserve_vec(capacity, storage)?;
    let mut target_count = 0_u64;
    for expected_ordinal in 0..count {
        let ordinal = cursor.u32()?;
        let lsn = cursor.u64()?;
        if u64::from(ordinal) != expected_ordinal
            || expected_start_lsn.checked_add(expected_ordinal) != Some(lsn)
        {
            return Err(CanonicalRedoWireDenial::InvalidRecordOrder.into());
        }
        let targets = decode_targets(
            &mut cursor,
            &mut target_count,
            maximum_targets,
            &mut distinct,
            storage,
        )?;
        let record_bytes = cursor.field()?;
        if record_bytes.is_empty() {
            return Err(CanonicalRedoWireDenial::MalformedMember.into());
        }
        records.push(CanonicalRedoWireRecord {
            ordinal,
            lsn,
            targets,
            bytes: copy_box(record_bytes, storage)?,
        });
    }
    let projection = PersistedPhysicalRecoveryProjection::decode_with_storage(
        cursor.field()?,
        projection_limits,
        format,
        storage,
    )?;
    cursor.require_end()?;
    validate_projection(&records, &projection)?;
    Ok((records.into_boxed_slice(), projection))
}

impl CanonicalRedoWireRecord {
    pub fn into_parts(self) -> (u32, u64, Box<[CanonicalRedoTarget]>, Box<[u8]>) {
        (self.ordinal, self.lsn, self.targets, self.bytes)
    }
}

fn validate_projection(
    records: &[CanonicalRedoWireRecord],
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), CanonicalRedoWireDenial> {
    let frames = projection
        .frames()
        .ok_or(CanonicalRedoWireDenial::InvalidRecoveryProjection)?;
    for target in records.iter().flat_map(|record| record.targets.iter()) {
        let matches = frames
            .iter()
            .filter(|frame| materialization_matches(target, frame))
            .count();
        if matches != 1 {
            return Err(CanonicalRedoWireDenial::InvalidRecoveryProjection);
        }
    }
    if frames.iter().any(|frame| {
        !records
            .iter()
            .flat_map(|record| record.targets.iter())
            .any(|target| materialization_matches(target, frame))
    }) {
        return Err(CanonicalRedoWireDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

fn materialization_matches(
    target: &CanonicalRedoTarget,
    frame: &crate::PersistedPhysicalRecoveryFrame,
) -> bool {
    let identity_matches = match (target.identity(), frame.subject()) {
        (
            CanonicalRedoTargetIdentity::InlinePage {
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
            CanonicalRedoTargetIdentity::ExtentChunk {
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
    fn require_count_backing(
        &self,
        count: u64,
        minimum_bytes: usize,
    ) -> Result<(), CanonicalRedoWireDenial> {
        let required = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(minimum_bytes));
        if required.is_none_or(|required| required > self.remaining.len()) {
            return Err(CanonicalRedoWireDenial::MalformedMember);
        }
        Ok(())
    }
    fn byte(&mut self) -> Result<u8, CanonicalRedoWireDenial> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, CanonicalRedoWireDenial> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, CanonicalRedoWireDenial> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], CanonicalRedoWireDenial> {
        self.take(N)?
            .try_into()
            .map_err(|_| CanonicalRedoWireDenial::MalformedMember)
    }
    fn field(&mut self) -> Result<&'a [u8], CanonicalRedoWireDenial> {
        let len =
            usize::try_from(self.u64()?).map_err(|_| CanonicalRedoWireDenial::MalformedMember)?;
        self.take(len)
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8], CanonicalRedoWireDenial> {
        let (head, tail) = self
            .remaining
            .split_at_checked(len)
            .ok_or(CanonicalRedoWireDenial::MalformedMember)?;
        self.remaining = tail;
        Ok(head)
    }
    fn require_end(self) -> Result<(), CanonicalRedoWireDenial> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(CanonicalRedoWireDenial::MalformedMember)
        }
    }
}
