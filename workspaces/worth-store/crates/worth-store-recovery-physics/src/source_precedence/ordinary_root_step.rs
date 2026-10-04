//! Exact, bounded topology transition for one C.9-admitted non-release root member.
//! The caller must still independently read both rooted inventories from media.

use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding, ExtentArenaId,
    ExtentArenaRange, FreeSpaceKey, IndexedThroughBlobPublication,
    PersistedPhysicalRecoveryOperation as Semantic, PersistedPhysicalRecoveryProjection,
    PersistedRecordIdentity, PhysicalInventoryTranscriptBuilderV1, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
    SegmentPageKey,
};
use worth_store_wal::WalLsnRange;

use super::ReleasedInventoryView;
use crate::{AdmittedRootStepMemberView, PhysicalRedoGroupBinding, RecoveryOperationFate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryRootStepDenial {
    BoundExceeded,
    InvalidSource,
    InvalidResult,
    InvalidMember,
    InvalidDelta,
}

/// A checked edge in an ordered root chain, not a Blob lifecycle certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedOrdinaryRootStep {
    source: PhysicalInventoryTranscriptV1,
    result: PhysicalInventoryTranscriptV1,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    redo_sha256: [u8; 32],
    lsn_range: Option<WalLsnRange>,
    scratch_bytes: u64,
}

/// Store's independently decoded WAL observation. This is data, not a C.9
/// admission token; `recheck_actual_media` binds it to one already-verified
/// C.8 step and replays the inventory predicate over Store-read media.
#[derive(Clone, Copy)]
pub struct ObservedOrdinaryRootMember<'a> {
    pub operation: [u8; 32],
    pub group: PhysicalRedoGroupBinding,
    pub fate: RecoveryOperationFate,
    pub canonical_redo_sha256: [u8; 32],
    pub lsn_range: WalLsnRange,
    pub projection: &'a PersistedPhysicalRecoveryProjection,
}

impl VerifiedOrdinaryRootStep {
    /// Additional peak used by this same actual-media recheck, excluding its
    /// borrowed source/result inventories and the decoded WAL projection.
    /// A Store caller can admit this amount in its native recovery pool before
    /// entering the allocation-owning predicate.
    pub fn maximum_recheck_heap_bytes(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        projection: &PersistedPhysicalRecoveryProjection,
        maximum_entries: u64,
    ) -> Option<u64> {
        scratch_charge(source, result, projection, maximum_entries)
    }
    /// An early C.9 member view lets PageAdmission prove a historical root
    /// chain before the later immutable redo plan can be constructed.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_preplanning(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        member: AdmittedRootStepMemberView<'_>,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, OrdinaryRootStepDenial> {
        Self::check(
            source,
            result,
            Some(member.lsn_range()),
            member.operation(),
            member.group(),
            member.fate(),
            member.canonical_redo_sha256(),
            member.materialization(),
            format,
            maximum_entries,
            maximum_scratch_bytes,
        )
    }

    /// Store must form `observed` from the exact uniquely sampled durable WAL
    /// member (including its independently decoded projection) and supply
    /// separately rewalked source/result inventories. No caller may use this
    /// method to mint a step without the existing private C.8 token.
    pub fn recheck_actual_media(
        self,
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        observed: ObservedOrdinaryRootMember<'_>,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<u64, OrdinaryRootStepDenial> {
        if self.operation != observed.operation
            || self.group != observed.group
            || self.fate != observed.fate
            || self.redo_sha256 != observed.canonical_redo_sha256
            || self.lsn_range != Some(observed.lsn_range)
        {
            return Err(OrdinaryRootStepDenial::InvalidMember);
        }
        let checked = Self::check(
            source,
            result,
            Some(observed.lsn_range),
            observed.operation,
            observed.group,
            observed.fate,
            observed.canonical_redo_sha256,
            observed.projection,
            format,
            maximum_entries,
            maximum_scratch_bytes,
        )?;
        if checked.source != self.source || checked.result != self.result {
            return Err(OrdinaryRootStepDenial::InvalidDelta);
        }
        Ok(checked.scratch_bytes)
    }

    #[allow(clippy::too_many_arguments)]
    fn check(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        lsn_range: Option<WalLsnRange>,
        operation: [u8; 32],
        group: PhysicalRedoGroupBinding,
        fate: RecoveryOperationFate,
        redo_sha256: [u8; 32],
        projection: &PersistedPhysicalRecoveryProjection,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, OrdinaryRootStepDenial> {
        use OrdinaryRootStepDenial as Denial;
        if maximum_entries == 0 || maximum_scratch_bytes == 0 {
            return Err(Denial::BoundExceeded);
        }
        if matches!(fate, RecoveryOperationFate::ProvenNoEffect)
            || matches!(projection.operation(), Semantic::RecordsDropped { .. })
            || projection.source_root_generation() != source.root.generation()
            || source.root.generation().checked_add(1) != Some(result.root.generation())
            || result.root.generation() != result.free.generation()
        {
            return Err(Denial::InvalidMember);
        }
        let scratch_bytes = scratch_charge(source, result, projection, maximum_entries)
            .filter(|charge| *charge <= maximum_scratch_bytes)
            .ok_or(Denial::BoundExceeded)?;
        let source_topology =
            transcript(source, format, maximum_entries).map_err(|_| Denial::InvalidSource)?;
        let result_topology =
            transcript(result, format, maximum_entries).map_err(|_| Denial::InvalidResult)?;
        if !root::root_matches(source, result, projection)
            || !delta::routes_match(source, result, projection)?
            || !delta::segments_match(source, result, projection)?
            || !delta::free_matches(source, result, projection, maximum_entries)?
        {
            return Err(Denial::InvalidDelta);
        }
        Ok(Self {
            source: source_topology,
            result: result_topology,
            operation,
            group,
            fate,
            redo_sha256,
            lsn_range,
            scratch_bytes,
        })
    }

    pub const fn source_topology(self) -> PhysicalInventoryTranscriptV1 {
        self.source
    }
    pub const fn result_topology(self) -> PhysicalInventoryTranscriptV1 {
        self.result
    }
    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn redo_sha256(self) -> [u8; 32] {
        self.redo_sha256
    }
    pub const fn lsn_range(self) -> Option<WalLsnRange> {
        self.lsn_range
    }
    pub const fn scratch_bytes(self) -> u64 {
        self.scratch_bytes
    }
}

pub(super) fn transcript(
    view: ReleasedInventoryView<'_>,
    format: PhysicalRecordFormatDeclaration,
    limit: u64,
) -> Result<PhysicalInventoryTranscriptV1, ()> {
    let mut builder =
        PhysicalInventoryTranscriptBuilderV1::new(view.root, view.free, format, limit)
            .map_err(|_| ())?;
    for route in view.routes {
        builder.include_route(*route).map_err(|_| ())?;
    }
    for segment in view.segments {
        builder.include_segment(*segment).map_err(|_| ())?;
    }
    for free in view.free_entries {
        builder.include_free(*free).map_err(|_| ())?;
    }
    builder.finish().map_err(|_| ())
}

fn scratch_charge(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projection: &PersistedPhysicalRecoveryProjection,
    limit: u64,
) -> Option<u64> {
    let counts = [
        source.routes.len(),
        source.segments.len(),
        source.free_entries.len(),
        result.routes.len(),
        result.segments.len(),
        result.free_entries.len(),
        projection.placements().len(),
        projection.segment_updates().len(),
        projection.root_state().inline_allocations().len(),
        match projection.operation() {
            Semantic::DerivedDirectory {
                retirement: Some(retirement),
                ..
            } => retirement.dropped_records().len(),
            _ => 0,
        },
    ];
    if counts.iter().any(|count| *count as u64 > limit) {
        return None;
    }
    let range_count = projection.placements().len().checked_mul(2)?;
    let total = counts
        .into_iter()
        .try_fold(range_count as u64, |sum, count| {
            sum.checked_add(count as u64)
        })?;
    // Charge the observed cardinality, not the caller's admitted ceiling:
    // Store may supply a large ceiling for a tiny actual root step. The 512
    // byte multiplier covers three ordered maps, route vector, arena splits,
    // and both transcript builders. No allocation precedes this charge.
    total.checked_mul(512)?.checked_add(16 * 1024)
}

#[path = "ordinary_root_step/delta.rs"]
mod delta;
#[path = "ordinary_root_step/root.rs"]
mod root;

#[cfg(test)]
#[path = "ordinary_root_step/tests.rs"]
mod tests;
