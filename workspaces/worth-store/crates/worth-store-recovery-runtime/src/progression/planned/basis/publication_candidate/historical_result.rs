//! C.8 adapter for the shared checked V3 transition. The source and result
//! inventories are independently admitted before this call; Store rewalks
//! physical media and executes the same predicate before a Serving seal.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedReleasedV3InventoryTransition,
    VerifiedSelectedReleaseHeadReplayV14,
};

use super::RecoverySelectedSourceInventory;

#[allow(clippy::too_many_arguments)]
pub(crate) fn verified_historical_release_transition(
    source_root: &DurablePhysicalRootManifest,
    source: &RecoverySelectedSourceInventory,
    source_routes: &[CurrentPhysicalRecordPlacement],
    result_root: &DurablePhysicalRootManifest,
    result: &RecoverySelectedSourceInventory,
    result_routes: &[CurrentPhysicalRecordPlacement],
    dropped: &[PersistedRecordIdentity],
    projected: &[CurrentPhysicalRecordPlacement],
    head_replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
    directory_replacement: Option<
        &worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement,
    >,
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
) -> Option<(VerifiedReleasedV3InventoryTransition, u64)> {
    let segment_width = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
    let result_requested = (result.segment_pages.len() as u64).checked_mul(segment_width)?;
    let source_requested = (source.segment_pages.len() as u64).checked_mul(segment_width)?;
    if source_requested.checked_add(result_requested)? > maximum_scratch_bytes {
        return None;
    }
    let mut source_segments = Vec::new();
    source_segments
        .try_reserve_exact(source.segment_pages.len())
        .ok()?;
    source_segments.extend(source.segment_pages.values().map(|page| page.entry));
    let source_allocated = (source_segments.capacity() as u64).checked_mul(segment_width)?;
    if source_allocated.checked_add(result_requested)? > maximum_scratch_bytes {
        return None;
    }
    let mut result_segments = Vec::new();
    result_segments
        .try_reserve_exact(result.segment_pages.len())
        .ok()?;
    result_segments.extend(result.segment_pages.values().map(|page| page.entry));
    let input_scratch = (source_segments.capacity() as u64)
        .checked_add(result_segments.capacity() as u64)?
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    let matcher_budget = maximum_scratch_bytes.checked_sub(input_scratch)?;
    let source_view = ReleasedInventoryView::new(
        source_root,
        &source.free_space,
        source_routes,
        &source_segments,
        &source.free_entries,
    );
    let result_view = ReleasedInventoryView::new(
        result_root,
        &result.free_space,
        result_routes,
        &result_segments,
        &result.free_entries,
    );
    let transition = match head_replay {
        Some(replay) => VerifiedReleasedV3InventoryTransition::admit_with_head_replay(
            source_view,
            result_view,
            dropped,
            projected,
            replay,
            format,
            maximum_entries,
            matcher_budget,
            directory_replacement,
        ),
        None => VerifiedReleasedV3InventoryTransition::admit(
            source_view,
            result_view,
            dropped,
            projected,
            format,
            maximum_entries,
            matcher_budget,
            directory_replacement,
        ),
    }
    .ok()?;
    Some((transition, input_scratch))
}

#[cfg(test)]
#[path = "historical_result/tests.rs"]
mod tests;
