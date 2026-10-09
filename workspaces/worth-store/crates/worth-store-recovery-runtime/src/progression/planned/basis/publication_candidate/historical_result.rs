//! C.8 adapter for the shared checked V3 transition. The source and result
//! inventories are independently admitted before this call; Store rewalks
//! physical media and executes the same predicate before a Serving seal.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    ExceededRootHistoryBound, ReleasedInventoryParts, VerifiedReleasedV3InventoryTransition,
    VerifiedSelectedReleaseHeadReplayV14,
};

use super::RecoverySelectedSourceInventory;

/// The transition and the scratch its laid-out segments hold. A refusal
/// names the bound it ran past, in the caller's `maximum_scratch_bytes`, if
/// that is why; `None` is a transition the media does not hold.
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
    VerifiedReleasedV3InventoryTransition::admit_laid_out(
        ReleasedInventoryParts {
            root: source_root,
            free: &source.free_space,
            routes: source_routes,
            segments: source.segment_pages.values().map(|page| page.entry),
            free_entries: &source.free_entries,
        },
        ReleasedInventoryParts {
            root: result_root,
            free: &result.free_space,
            routes: result_routes,
            segments: result.segment_pages.values().map(|page| page.entry),
            free_entries: &result.free_entries,
        },
        dropped,
        projected,
        head_replay,
        directory_replacement,
        format,
        maximum_entries,
        maximum_scratch_bytes,
    )
    .map_err(|denial| denial.exceeded_bound())
}

#[cfg(test)]
#[path = "historical_result/tests.rs"]
mod tests;
