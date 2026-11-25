//! Exact byte-for-byte adoption check for an interrupted recovery's own
//! canonical `build_new` successor candidate when it differs structurally from
//! a live writer's incremental copy-on-write candidate (e.g. rebuilt trees with
//! unchanged membership).
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};

use super::{build_new, inventory, CandidateBuildDenial};
use crate::progression::planned::basis::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate, RecoverySelectedSourceInventory,
};
use crate::progression::planned::PlanningResidentAllowance;

pub(super) fn match_observed(
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    format: PhysicalRecordFormatDeclaration,
    observed: &RecoveryObservedSuccessorCandidate,
    maintenance: bool,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Option<(DurablePhysicalRootManifest, u64)>, CandidateBuildDenial> {
    let baseline_used = allowance.used();
    let (expected_root, mut expected_build) = build_new(
        base,
        source,
        final_inventory,
        format,
        maintenance,
        allowance,
    )?;
    expected_build
        .artifacts
        .sort_unstable_by_key(|artifact| artifact.artifact);
    let mut scratch_bytes = 0u64;
    for artifact in &expected_build.artifacts {
        scratch_bytes = scratch_bytes.max(artifact.bytes.len() as u64);
    }
    let matches = expected_root == observed.root
        && expected_build.artifacts.len() == observed.artifacts.len()
        && expected_build
            .artifacts
            .iter()
            .zip(observed.artifacts.iter())
            .all(|(e, o)| e.artifact == o.artifact && e.bytes.as_ref() == o.bytes.as_ref());
    drop(expected_build);
    allowance.release_to_baseline(baseline_used);
    Ok(matches.then_some((expected_root, scratch_bytes)))
}
