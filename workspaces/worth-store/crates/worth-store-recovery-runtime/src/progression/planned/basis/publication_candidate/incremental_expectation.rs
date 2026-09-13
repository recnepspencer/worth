use worth_store_physical_format::{
    durable_artifact_checksum, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordArtifactFile,
};

use super::{inventory, CandidateBuildDenial};
use crate::progression::planned::basis::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate, RecoverySelectedSourceInventory,
};

mod canonical_candidate_match;
mod free_space;
mod root_routing;
mod segment_membership;

use canonical_candidate_match::CanonicalCandidateMatch;

pub(super) fn derive(
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    format: PhysicalRecordFormatDeclaration,
    observed: &RecoveryObservedSuccessorCandidate,
) -> Result<(DurablePhysicalRootManifest, u64), CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let mut matcher = CanonicalCandidateMatch::new(format, generation, &observed.artifacts)?;
    let (segment_root, next_segment_block) =
        segment_membership::derive(&mut matcher, base, source, final_inventory)?;
    let free = free_space::derive(&mut matcher, base, source, final_inventory)?;
    let free_bytes = free.encode(format);
    let free_checksum = durable_artifact_checksum(&free_bytes);
    matcher.match_artifact(
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_bytes,
    )?;
    let (routing_root, next_block) = root_routing::derive(&mut matcher, base, final_inventory)?;
    let root = DurablePhysicalRootManifest::builder(
        generation,
        selected.tree_identity(),
        final_inventory.capacity,
        free_checksum,
    )
    .record_count(final_inventory.placements.len() as u64)
    .next_block(next_block)
    .next_segment_block(next_segment_block)
    .routing_root(routing_root)
    .segment_root(segment_root)
    .free_space_root(free.root())
    .last_inline_record(
        final_inventory
            .last_inline_record
            .or(selected.last_inline_record()),
    )
    .last_inline_segment(
        final_inventory
            .last_inline_segment
            .or(selected.last_inline_segment()),
    )
    .admit()
    .ok_or(CandidateBuildDenial::Invalid)?;
    matcher.match_artifact(
        RecordArtifactFile::RootManifest { generation },
        root.encode(format),
    )?;
    let comparison_scratch_bytes = matcher.finish()?;
    Ok((root, comparison_scratch_bytes))
}
