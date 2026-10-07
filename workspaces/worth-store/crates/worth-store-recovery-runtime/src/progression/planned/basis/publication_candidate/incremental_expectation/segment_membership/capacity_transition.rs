use worth_store_physical_format::SegmentManifestBlockReference;

use super::super::super::{inventory, CandidateBuildDenial};
use super::super::CanonicalCandidateMatch;
use super::{build_tree, Update};
use crate::progression::planned::basis::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};
use crate::progression::planned::PlanningResidentAllowance;

/// A capacity change rewrites every selected membership branch; there is no
/// unchanged-child shortcut even when its key range has no updates.
pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    updates: &[Update],
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Option<SegmentManifestBlockReference>, u64), CandidateBuildDenial> {
    build_tree(
        matcher,
        base,
        source,
        final_inventory,
        updates,
        true,
        allowance,
    )
}
