//! A bounded canonical transcript of C.8's verified source and derived
//! candidate. Store independently rewalks both physical inventories before
//! converting a pending WAL release into Serving authority.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_recovery_physics::{ReleasedInventoryView, VerifiedReleasedV3InventoryTransition};

use super::super::RecoveryReleaseTopologyProof;
use super::{
    inventory::FinalInventory, CandidateBuildDenial, RecoveryPublicationCandidateArtifact,
};
use crate::progression::planned::{PlanningMemoryDenial, PlanningResidentAllowance};
use crate::progression::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};

#[allow(clippy::too_many_arguments)]
pub(super) fn mint(
    base: &RecoveryBaseImagePlan,
    source_root: &DurablePhysicalRootManifest,
    source: &RecoverySelectedSourceInventory,
    source_routes: &[CurrentPhysicalRecordPlacement],
    published_root: &DurablePhysicalRootManifest,
    artifacts: &[RecoveryPublicationCandidateArtifact],
    published: &FinalInventory,
    format: PhysicalRecordFormatDeclaration,
    dropped: &[PersistedRecordIdentity],
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(RecoveryReleaseTopologyProof, u64), CandidateBuildDenial> {
    let published_free = artifacts
        .iter()
        .find(|artifact| {
            artifact.artifact()
                == RecordArtifactFile::FreeSpaceManifest {
                    generation: published_root.generation(),
                }
        })
        .and_then(|artifact| {
            DurableFreeSpaceManifestHeader::decode(artifact.bytes(), published_root.node_capacity())
                .ok()
        })
        .filter(|(_, observed_format)| *observed_format == format)
        .map(|(header, _)| header)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut source_segments = allowance.reserve(source.segment_pages.len())?;
    source_segments.extend(source.segment_pages.values().map(|page| page.entry));
    let mut projected = allowance.reserve(
        base.actions()
            .iter()
            .filter(|action| action.is_projected())
            .count(),
    )?;
    projected.extend(
        base.actions()
            .iter()
            .copied()
            .filter(|action| action.is_projected())
            .map(|action| action.placement()),
    );
    projected.sort_unstable_by_key(|route| route.record());
    let input_scratch = PlanningResidentAllowance::vector_bytes(&source_segments)?
        .checked_add(PlanningResidentAllowance::vector_bytes(&projected)?)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let source_view = ReleasedInventoryView::new(
        source_root,
        &source.free_space,
        source_routes,
        &source_segments,
        &source.free_entries,
    );
    let result_view = ReleasedInventoryView::new(
        published_root,
        &published_free,
        &published.placements,
        &published.segments,
        &published.free,
    );
    let matcher_budget = VerifiedReleasedV3InventoryTransition::maximum_construction_heap_bytes(
        source_view,
        result_view,
        &projected,
        maximum_entries,
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let total_scratch = input_scratch
        .checked_add(matcher_budget)
        .ok_or(CandidateBuildDenial::Invalid)?;
    if total_scratch > maximum_scratch_bytes {
        return Err(CandidateBuildDenial::StagingBytes {
            observed: total_scratch,
        });
    }
    // This window stays charged alongside both inventories and the candidate.
    allowance.retain(matcher_budget)?;
    let transition = match base.release_head_replay() {
        Some(replay) => VerifiedReleasedV3InventoryTransition::admit_with_head_replay(
            source_view,
            result_view,
            dropped,
            &projected,
            replay,
            format,
            maximum_entries,
            matcher_budget,
        ),
        None => VerifiedReleasedV3InventoryTransition::admit(
            source_view,
            result_view,
            dropped,
            &projected,
            format,
            maximum_entries,
            matcher_budget,
        ),
    }
    .map_err(|denial| match denial {
        worth_store_recovery_physics::ReleasedV3InventoryTransitionDenial::Allocation {
            requested_bytes,
            cause,
        } => CandidateBuildDenial::Memory(PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        }),
        _ => CandidateBuildDenial::Invalid,
    })?;
    let transition_scratch = transition.scratch_bytes();
    let retained = transition
        .owned_heap_bytes()
        .ok_or(CandidateBuildDenial::Invalid)?;
    // All Physics scratch has dropped; the proof's actual Box backing remains.
    allowance.release(matcher_budget);
    allowance.retain(retained)?;
    drop((projected, source_segments));
    allowance.release(input_scratch);
    Ok((
        RecoveryReleaseTopologyProof {
            source_free: source.free_space.clone(),
            published_free,
            transition,
        },
        input_scratch
            .checked_add(transition_scratch)
            .ok_or(CandidateBuildDenial::Invalid)?,
    ))
}
