//! C.8 adapter for the shared checked V3 transition. The source and result
//! inventories are independently admitted before this call; Store rewalks
//! physical media and executes the same predicate before a Serving seal.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    ExceededRootHistoryBound, ReleasedInventoryView, RootHistoryBound,
    VerifiedReleasedV3InventoryTransition, VerifiedSelectedReleaseHeadReplayV14,
};

use super::RecoverySelectedSourceInventory;

/// The transition and the scratch its segment views took. A refusal names
/// the bound it ran past, in the caller's `maximum_scratch_bytes`, if that is
/// why; `None` is a transition the media does not hold.
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
) -> Result<(VerifiedReleasedV3InventoryTransition, u64), Option<ExceededRootHistoryBound>> {
    let past = |observed| ExceededRootHistoryBound {
        bound: RootHistoryBound::ScratchBytes,
        observed,
        admitted: maximum_scratch_bytes,
    };
    // Two segment views of these lengths, held together. A size past every
    // count is past every bound.
    let pair = |source: usize, result: usize| {
        let width = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
        let bytes = (source as u64)
            .checked_add(result as u64)
            .and_then(|count| count.checked_mul(width))
            .unwrap_or(u64::MAX);
        if bytes > maximum_scratch_bytes {
            return Err(Some(past(bytes)));
        }
        Ok(bytes)
    };
    pair(source.segment_pages.len(), result.segment_pages.len())?;
    let mut source_segments = Vec::new();
    source_segments
        .try_reserve_exact(source.segment_pages.len())
        .map_err(|_| None)?;
    source_segments.extend(source.segment_pages.values().map(|page| page.entry));
    pair(source_segments.capacity(), result.segment_pages.len())?;
    let mut result_segments = Vec::new();
    result_segments
        .try_reserve_exact(result.segment_pages.len())
        .map_err(|_| None)?;
    result_segments.extend(result.segment_pages.values().map(|page| page.entry));
    let input_scratch = pair(source_segments.capacity(), result_segments.capacity())?;
    let matcher_budget = maximum_scratch_bytes - input_scratch;
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
    .map_err(|denial| {
        // Physics had `matcher_budget`; the segment views hold the rest.
        let exceeded = denial.exceeded_bound()?;
        Some(match exceeded.bound {
            RootHistoryBound::ScratchBytes => past(exceeded.observed.saturating_add(input_scratch)),
            RootHistoryBound::Entries => exceeded,
        })
    })?;
    Ok((transition, input_scratch))
}

#[cfg(test)]
#[path = "historical_result/tests.rs"]
mod tests;
