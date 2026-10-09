use worth_store_physical_format::{
    durable_artifact_checksum, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordArtifactFile,
};

use super::{inventory, CandidateBuildDenial};
use crate::progression::planned::basis::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate, RecoverySelectedSourceInventory,
};
use crate::progression::planned::PlanningResidentAllowance;

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
    maintenance: bool,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(DurablePhysicalRootManifest, u64), CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let mut matcher = CanonicalCandidateMatch::new(format, generation, &observed.artifacts)?;
    let (segment_root, next_segment_block) =
        segment_membership::derive(&mut matcher, base, source, final_inventory, allowance)?;
    let free = free_space::derive(&mut matcher, base, source, final_inventory, allowance)?;
    let free_bytes = super::encoding::free_header(&free, format, allowance)?;
    let free_checksum = durable_artifact_checksum(&free_bytes);
    matcher.match_artifact(
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_bytes,
        allowance,
    )?;
    let (routing_root, next_block) =
        root_routing::derive(&mut matcher, base, final_inventory, allowance)?;
    if let Some(replay) = base.release_head_replay() {
        for write in replay.effect().node_writes() {
            let reference = write.reference();
            let mut expected = allowance.reserve::<u8>(write.frame().len())?;
            expected.extend_from_slice(write.frame());
            matcher.match_artifact(
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                expected,
                allowance,
            )?;
        }
    }
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
    .release_custody_head_root(super::release_head::result_fields(base).0)
    .next_release_custody_head_block(super::release_head::result_fields(base).1)
    .tier_epoch_anchor(base.tier_epoch_anchor())
    .latest_blob_publication(base.latest_blob_publication())
    .latest_blob_quarantine(base.latest_blob_quarantine())
    .derived_family_directory(base.derived_family_directory())
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
    let root = if maintenance
        || selected.requires_maintenance_protocol()
        || observed.root.requires_maintenance_protocol()
    {
        root.with_maintenance_protocol()
    } else {
        root
    };
    let root_bytes = super::encoding::root_manifest(&root, format, allowance)?;
    matcher.match_artifact(
        RecordArtifactFile::RootManifest { generation },
        root_bytes,
        allowance,
    )?;
    let comparison_scratch_bytes = matcher.finish()?;
    Ok((root, comparison_scratch_bytes))
}
