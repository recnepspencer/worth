use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::{PlanningMemoryDenial, PlanningResidentAllowance};
use super::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate,
    RecoveryPublicationCandidateArtifact, RecoverySelectedSourceInventory,
};
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;

mod adoption;
mod cost;
pub(super) mod encoding;
mod frame_storage;
mod frontier;
mod historical_result;
mod incremental_expectation;
mod inventory;
mod protocol;
mod release_head;
mod topology_transcript;
mod tree;

pub(crate) use historical_result::verified_historical_release_transition;

pub(super) struct RecoveryCandidateBasis {
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) referenced_artifacts: Box<[RecordArtifactFile]>,
    pub(super) artifacts: Box<[RecoveryPublicationCandidateArtifact]>,
    pub(super) materialization_cost: CandidateMaterializationCost,
    pub(super) staged_current_selector: DurableRootSelector,
    pub(super) release_topology: Option<super::RecoveryReleaseTopologyProof>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CandidateMaterializationCost {
    comparison_scratch_bytes: u64,
    publication_bytes: u64,
}

impl CandidateMaterializationCost {
    pub(crate) const fn comparison_scratch_bytes(self) -> u64 {
        self.comparison_scratch_bytes
    }

    pub(crate) const fn publication_bytes(self) -> u64 {
        self.publication_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CandidateBuildDenial {
    Memory(PlanningMemoryDenial),
    StagingBytes { observed: u64 },
    SuccessorCandidate(PhysicalRecoverySuccessorCandidateDenial),
    Invalid,
}

impl From<PlanningMemoryDenial> for CandidateBuildDenial {
    fn from(denial: PlanningMemoryDenial) -> Self {
        Self::Memory(denial)
    }
}

pub(super) fn build(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    source_routes: &[worth_store_physical_format::CurrentPhysicalRecordPlacement],
    observed_successor: Option<RecoveryObservedSuccessorCandidate>,
    format: PhysicalRecordFormatDeclaration,
    publication: u64,
    maintenance: bool,
    verified_drops: &[worth_store_physical_format::PersistedRecordIdentity],
    maximum_manifest_entries: u64,
    maximum_staging_bytes: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<RecoveryCandidateBasis, CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let final_inventory = inventory::finalize(
        source,
        base.actions(),
        base.segment_updates(),
        base.root_states(),
        selected.node_capacity(),
        generation,
        maximum_staging_bytes / 40,
        allowance,
    )?;
    let mut comparison_scratch_bytes = 0;
    let (root, mut build, referenced_artifacts) = match observed_successor {
        Some(observed) => {
            adoption::admit_observed(base, source, &final_inventory, &observed, allowance)?;
            let (expected_root, scratch_bytes) = incremental_expectation::derive(
                base,
                source,
                &final_inventory,
                format,
                &observed,
                maintenance,
                allowance,
            )?;
            debug_assert_eq!(expected_root, observed.root);
            comparison_scratch_bytes = scratch_bytes;
            observed_build(format, observed, allowance)?
        }
        None => {
            let (root, build) = build_new(
                base,
                source,
                &final_inventory,
                format,
                maintenance,
                allowance,
            )?;
            let referenced_artifacts = topology_artifacts(&build.artifacts, build.allowance)?;
            (root, build, referenced_artifacts)
        }
    };
    let staged_current_selector = protocol::push_candidates(
        &mut build,
        store,
        format,
        base.selected_selector(),
        generation,
        publication,
    )?;
    build
        .artifacts
        .sort_unstable_by_key(|artifact| artifact.artifact);
    let publication_bytes =
        cost::candidate_materialization_bytes(&root, &referenced_artifacts, &build.artifacts)?;
    let release_topology = if !verified_drops.is_empty() {
        let (topology, scratch) = topology_transcript::mint(
            base,
            base.selected_root(),
            source,
            source_routes,
            &root,
            &build.artifacts,
            &final_inventory,
            format,
            verified_drops,
            maximum_manifest_entries,
            maximum_staging_bytes,
            build.allowance,
        )?;
        comparison_scratch_bytes = comparison_scratch_bytes.max(scratch);
        Some(topology)
    } else {
        None
    };
    final_inventory.release(build.allowance)?;
    let artifacts = build.allowance.into_box(build.artifacts)?;
    Ok(RecoveryCandidateBasis {
        root,
        referenced_artifacts,
        artifacts,
        materialization_cost: CandidateMaterializationCost {
            comparison_scratch_bytes,
            publication_bytes,
        },
        staged_current_selector,
        release_topology,
    })
}

fn observed_build<'a>(
    format: PhysicalRecordFormatDeclaration,
    observed: RecoveryObservedSuccessorCandidate,
    allowance: &'a mut PlanningResidentAllowance,
) -> Result<
    (
        DurablePhysicalRootManifest,
        CandidateBuild<'a>,
        Box<[RecordArtifactFile]>,
    ),
    CandidateBuildDenial,
> {
    let mut build = CandidateBuild {
        format,
        artifacts: allowance.reserve(
            observed
                .artifacts
                .len()
                .checked_add(3)
                .ok_or(CandidateBuildDenial::Invalid)?,
        )?,
        allowance,
    };
    let RecoveryObservedSuccessorCandidate {
        root,
        referenced_artifacts,
        artifacts,
        placements,
        segment_entries,
        free_entries,
        free_space: _,
    } = observed;
    let discarded_bytes = PlanningResidentAllowance::slot_bytes::<
        worth_store_physical_format::CurrentPhysicalRecordPlacement,
    >(placements.len())?
    .checked_add(PlanningResidentAllowance::slot_bytes::<
        worth_store_physical_format::RecordSegmentPageManifestEntry,
    >(segment_entries.len())?)
    .and_then(|bytes| {
        bytes.checked_add(
            PlanningResidentAllowance::slot_bytes::<
                worth_store_physical_format::RecordFreeSpaceManifestEntry,
            >(free_entries.len())
            .ok()?,
        )
    })
    .ok_or(CandidateBuildDenial::Invalid)?;
    drop((placements, segment_entries, free_entries));
    build.allowance.release(discarded_bytes);
    let descriptor_bytes = PlanningResidentAllowance::slot_bytes::<
        super::RecoveryObservedCandidateArtifact,
    >(artifacts.len())?;
    for artifact in artifacts.into_vec() {
        build.push_owned(artifact.artifact, artifact.bytes)?;
    }
    build.allowance.release(descriptor_bytes);
    Ok((root, build, referenced_artifacts))
}

fn topology_artifacts(
    artifacts: &[RecoveryPublicationCandidateArtifact],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[RecordArtifactFile]>, CandidateBuildDenial> {
    let mut topology = allowance.reserve(artifacts.len())?;
    topology.extend(
        artifacts
            .iter()
            .map(RecoveryPublicationCandidateArtifact::artifact)
            .filter(|artifact| {
                matches!(
                    artifact,
                    RecordArtifactFile::RootManifest { .. }
                        | RecordArtifactFile::RootRoutingBlock { .. }
                        | RecordArtifactFile::ReleaseCustodyHeadBlock { .. }
                        | RecordArtifactFile::SegmentMembershipBlock { .. }
                        | RecordArtifactFile::FreeSpaceManifest { .. }
                        | RecordArtifactFile::FreeSpaceMembershipBlock { .. }
                )
            }),
    );
    topology.sort_unstable();
    Ok(allowance.into_box(topology)?)
}

fn build_new<'a>(
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    format: PhysicalRecordFormatDeclaration,
    maintenance: bool,
    allowance: &'a mut PlanningResidentAllowance,
) -> Result<(DurablePhysicalRootManifest, CandidateBuild<'a>), CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let mut build = CandidateBuild {
        format,
        artifacts: Vec::new(),
        allowance,
    };
    let (routing_root, next_block) = tree::root_routing(
        &mut build,
        &final_inventory.placements,
        selected.tree_identity(),
        generation,
        final_inventory.capacity,
        selected.next_block(),
    )?;
    let (segment_root, next_segment_block) = tree::segment_routing(
        &mut build,
        &final_inventory.segments,
        selected.tree_identity(),
        generation,
        final_inventory.capacity,
        selected.next_segment_block(),
    )?;
    let (free_root, next_free_block) = tree::free_space_routing(
        &mut build,
        &final_inventory.free,
        source.free_space.tree_identity(),
        generation,
        final_inventory.capacity,
        source.free_space.next_block(),
    )?;
    let free_space = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        generation,
        source.free_space.tree_identity(),
        final_inventory.capacity,
        source.free_space.segment_page_capacity(),
        final_inventory.free.len() as u64,
        final_inventory.next_segment,
        final_inventory.next_page,
        final_inventory.next_extent,
        final_inventory.next_arena,
        source.free_space.tier_epoch_start(),
        source.free_space.arena_capacity(),
        source.free_space.arena_alignment(),
        next_free_block,
        free_root,
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let free_bytes = encoding::free_header(&free_space, format, build.allowance)?;
    let free_checksum = durable_artifact_checksum(&free_bytes);
    build.push(
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_bytes,
    )?;
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
    .free_space_root(free_root)
    .release_custody_head_root(release_head::result_fields(base).0)
    .next_release_custody_head_block(release_head::result_fields(base).1)
    .latest_blob_publication(base.latest_blob_publication())
    .latest_blob_quarantine(base.latest_blob_quarantine())
    .tier_epoch_anchor(base.tier_epoch_anchor())
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
    let root = if maintenance || selected.requires_maintenance_protocol() {
        root.with_maintenance_protocol()
    } else {
        root
    };
    let root_bytes = encoding::root_manifest(&root, format, build.allowance)?;
    build.push(RecordArtifactFile::RootManifest { generation }, root_bytes)?;
    release_head::append_exact_writes(base, &mut build)?;
    Ok((root, build))
}

pub(super) struct CandidateBuild<'a> {
    format: PhysicalRecordFormatDeclaration,
    artifacts: Vec<RecoveryPublicationCandidateArtifact>,
    allowance: &'a mut PlanningResidentAllowance,
}
