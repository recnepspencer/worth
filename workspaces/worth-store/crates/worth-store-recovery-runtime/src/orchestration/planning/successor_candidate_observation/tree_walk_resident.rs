//! Resident charges for the three bounded successor metadata-tree traversals.
//! The format limit, not the candidate root's narrower capacity, bounds an
//! integrity decoder before the root-specific capacity check runs.

use std::mem::size_of;

use worth_store_physical_format::{
    maximum_current_root_entries, maximum_segment_manifest_pages, CurrentPhysicalRecordPlacement,
    FreeSpaceBlockReference, ManifestBlockReference, PhysicalRecordFormatDeclaration,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry, SegmentManifestBlockReference,
};

use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

fn overflow() -> PlanningMemoryDenial {
    PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX }
}

fn copies(
    count: usize,
    entry_bytes: usize,
    reference_bytes: usize,
    copies: u64,
) -> Result<u64, PlanningMemoryDenial> {
    let width = u64::try_from(entry_bytes.max(reference_bytes)).map_err(|_| overflow())?;
    u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(width))
        .and_then(|one| one.checked_mul(copies))
        .ok_or_else(overflow)
}

pub(super) fn root_projection_scratch(
    format: PhysicalRecordFormatDeclaration,
) -> Result<u64, PlanningMemoryDenial> {
    let count = usize::from(maximum_current_root_entries(format));
    let blocks = copies(
        count,
        size_of::<CurrentPhysicalRecordPlacement>(),
        size_of::<ManifestBlockReference>(),
        3,
    )?;
    // A leaf constructor checks physical-coordinate uniqueness with one
    // exact-count sorted Vec while decoded and copied blocks remain live.
    blocks
        .checked_add(PlanningResidentAllowance::slot_bytes::<
            worth_store_physical_format::RootRoutingCoordinateKey,
        >(count)?)
        .ok_or_else(overflow)
}

pub(super) fn segment_projection_scratch(
    format: PhysicalRecordFormatDeclaration,
) -> Result<u64, PlanningMemoryDenial> {
    let count = usize::try_from(maximum_segment_manifest_pages(format)).map_err(|_| overflow())?;
    copies(
        count,
        size_of::<RecordSegmentPageManifestEntry>(),
        size_of::<SegmentManifestBlockReference>(),
        2,
    )
}

pub(super) fn free_projection_scratch(
    format: PhysicalRecordFormatDeclaration,
) -> Result<u64, PlanningMemoryDenial> {
    copies(
        usize::from(maximum_current_root_entries(format)),
        size_of::<RecordFreeSpaceManifestEntry>(),
        size_of::<FreeSpaceBlockReference>(),
        2,
    )
}

/// A sorted, allocation-admitted duplicate guard for cold successor-tree
/// reconstruction. Lookup is O(log N); insertion shifts O(N) keys. The
/// manifest-entry budget bounds N, and no standard-library tree-node layout
/// assumption is needed to admit its actual Vec backing before insertion.
pub(super) struct VisitedNodes {
    keys: Vec<(u64, u64)>,
}

impl VisitedNodes {
    pub(super) const fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Duplicate detection precedes both growth and the artifact read.
    pub(super) fn insert(
        &mut self,
        key: (u64, u64),
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<bool, PlanningMemoryDenial> {
        let Err(position) = self.keys.binary_search(&key) else {
            return Ok(false);
        };
        allowance.grow(&mut self.keys, 1)?;
        self.keys.insert(position, key);
        Ok(true)
    }

    pub(super) fn release(
        self,
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<(), PlanningMemoryDenial> {
        let bytes = PlanningResidentAllowance::vector_bytes(&self.keys)?;
        drop(self);
        allowance.release(bytes);
        Ok(())
    }
}
