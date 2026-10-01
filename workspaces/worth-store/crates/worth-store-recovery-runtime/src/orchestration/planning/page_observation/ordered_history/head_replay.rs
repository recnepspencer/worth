//! Bind one historical released edge's WAL-chosen head path to its addressed
//! source-tree blocks while the bounded discovery cursor is still live.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    AdmittedRootStepMemberView, VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
    VerifiedSelectedReleaseHeadReplayV14,
};

use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;

#[allow(clippy::too_many_arguments)]
pub(super) fn admit_addressed_member(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    member: AdmittedRootStepMemberView<'_>,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    maximum_effect_bytes: u64,
) -> Option<VerifiedSelectedReleaseHeadReplayV14> {
    VerifiedSelectedReleaseHeadReplayV14::admit_addressed_member(
        member,
        source,
        result,
        format,
        maximum_effect_bytes,
        maximum_effect_bytes,
        |reference, maximum| {
            budget.consume(1).map_err(|_| ())?;
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|_| ())?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .ok()
}

pub(super) fn bind_edge(
    edge: &VerifiedReleasedRootEdge,
    replay: VerifiedSelectedReleaseHeadReplayV14,
    source: &DurablePhysicalRootManifest,
    result: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    remaining_additional_heap_bytes: u64,
) -> Option<VerifiedOrderedReleasedHeadReplayV14> {
    VerifiedOrderedReleasedHeadReplayV14::bind_edge(
        edge,
        replay,
        source,
        result,
        format,
        remaining_additional_heap_bytes,
    )
    .ok()
}
