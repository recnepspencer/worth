use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    durable_artifact_checksum, BootstrapCatalog, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, PhysicalRecordFormatDeclaration, RecordArtifactFile, RootSelectorIdentity,
    RootSelectorRole,
};

use super::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate,
    RecoveryPublicationCandidateArtifact, RecoverySelectedSourceInventory,
};
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;

mod adoption;
mod frontier;
mod incremental_expectation;
mod inventory;
mod tree;

pub(super) struct RecoveryCandidateBasis {
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) referenced_artifacts: Box<[RecordArtifactFile]>,
    pub(super) artifacts: Box<[RecoveryPublicationCandidateArtifact]>,
    pub(super) materialization_cost: CandidateMaterializationCost,
    pub(super) staged_current_selector: DurableRootSelector,
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
    SuccessorCandidate(PhysicalRecoverySuccessorCandidateDenial),
    Invalid,
}

pub(super) fn build(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    observed_successor: Option<RecoveryObservedSuccessorCandidate>,
    format: PhysicalRecordFormatDeclaration,
    publication: u64,
) -> Result<RecoveryCandidateBasis, CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let final_inventory = inventory::finalize(
        source,
        base.actions(),
        base.segment_updates(),
        base.root_states(),
        selected.node_capacity(),
    )?;
    let mut comparison_scratch_bytes = 0;
    let (root, mut build, referenced_artifacts) = match observed_successor {
        Some(observed) => {
            adoption::admit_observed(base, source, &final_inventory, &observed)?;
            let (expected_root, scratch_bytes) =
                incremental_expectation::derive(base, source, &final_inventory, format, &observed)?;
            debug_assert_eq!(expected_root, observed.root);
            comparison_scratch_bytes = scratch_bytes;
            observed_build(format, observed)?
        }
        None => {
            let (root, build) = build_new(base, source, &final_inventory, format)?;
            let referenced_artifacts = topology_artifacts(&build.artifacts);
            (root, build, referenced_artifacts)
        }
    };
    let staged_current_selector = push_protocol_candidates(
        &mut build,
        store,
        format,
        base.selected_selector(),
        generation,
        publication,
    )?;
    build.artifacts.sort_by_key(|artifact| artifact.artifact);
    let publication_bytes =
        candidate_materialization_bytes(&root, &referenced_artifacts, &build.artifacts)?;
    Ok(RecoveryCandidateBasis {
        root,
        referenced_artifacts,
        artifacts: build.artifacts.into_boxed_slice(),
        materialization_cost: CandidateMaterializationCost {
            comparison_scratch_bytes,
            publication_bytes,
        },
        staged_current_selector,
    })
}

fn candidate_materialization_bytes(
    _root: &DurablePhysicalRootManifest,
    referenced_artifacts: &[RecordArtifactFile],
    artifacts: &[RecoveryPublicationCandidateArtifact],
) -> Result<u64, CandidateBuildDenial> {
    let root_bytes = std::mem::size_of::<DurablePhysicalRootManifest>() as u64;
    let reference_bytes = (referenced_artifacts.len() as u64)
        .checked_mul(std::mem::size_of::<RecordArtifactFile>() as u64)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let descriptor_bytes = (artifacts.len() as u64)
        .checked_mul(std::mem::size_of::<RecoveryPublicationCandidateArtifact>() as u64)
        .ok_or(CandidateBuildDenial::Invalid)?;
    artifacts.iter().try_fold(
        root_bytes
            .checked_add(reference_bytes)
            .and_then(|bytes| bytes.checked_add(descriptor_bytes))
            .ok_or(CandidateBuildDenial::Invalid)?,
        |bytes, artifact| {
            bytes
                .checked_add(artifact.bytes.len() as u64)
                .ok_or(CandidateBuildDenial::Invalid)
        },
    )
}

fn observed_build(
    format: PhysicalRecordFormatDeclaration,
    observed: RecoveryObservedSuccessorCandidate,
) -> Result<
    (
        DurablePhysicalRootManifest,
        CandidateBuild,
        Box<[RecordArtifactFile]>,
    ),
    CandidateBuildDenial,
> {
    let mut build = CandidateBuild {
        format,
        artifacts: Vec::with_capacity(observed.artifacts.len() + 3),
    };
    let RecoveryObservedSuccessorCandidate {
        root,
        referenced_artifacts,
        artifacts,
        ..
    } = observed;
    for artifact in artifacts.into_vec() {
        build.push_owned(artifact.artifact, artifact.bytes)?;
    }
    Ok((root, build, referenced_artifacts))
}

fn topology_artifacts(
    artifacts: &[RecoveryPublicationCandidateArtifact],
) -> Box<[RecordArtifactFile]> {
    let mut topology = artifacts
        .iter()
        .map(RecoveryPublicationCandidateArtifact::artifact)
        .filter(|artifact| {
            matches!(
                artifact,
                RecordArtifactFile::RootManifest { .. }
                    | RecordArtifactFile::RootRoutingBlock { .. }
                    | RecordArtifactFile::SegmentMembershipBlock { .. }
                    | RecordArtifactFile::FreeSpaceManifest { .. }
                    | RecordArtifactFile::FreeSpaceMembershipBlock { .. }
            )
        })
        .collect::<Vec<_>>();
    topology.sort_unstable();
    topology.into_boxed_slice()
}

fn build_new(
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(DurablePhysicalRootManifest, CandidateBuild), CandidateBuildDenial> {
    let generation = base.destination_generation();
    let selected = base.selected_root();
    let mut build = CandidateBuild {
        format,
        artifacts: Vec::new(),
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
    let free_space = DurableFreeSpaceManifestHeader::new(
        generation,
        source.free_space.tree_identity(),
        final_inventory.capacity,
        source.free_space.segment_page_capacity(),
        final_inventory.free.len() as u64,
        final_inventory.next_segment,
        final_inventory.next_page,
        final_inventory.next_extent,
        next_free_block,
        free_root,
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let free_bytes = free_space.encode(format);
    build.push(
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_bytes.clone(),
    )?;
    let root = DurablePhysicalRootManifest::builder(
        generation,
        selected.tree_identity(),
        final_inventory.capacity,
        durable_artifact_checksum(&free_bytes),
    )
    .record_count(final_inventory.placements.len() as u64)
    .next_block(next_block)
    .next_segment_block(next_segment_block)
    .routing_root(routing_root)
    .segment_root(segment_root)
    .free_space_root(free_root)
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
    build.push(
        RecordArtifactFile::RootManifest { generation },
        root.encode(format),
    )?;
    Ok((root, build))
}

fn push_protocol_candidates(
    build: &mut CandidateBuild,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    selected: DurableRootSelector,
    generation: u64,
    publication: u64,
) -> Result<DurableRootSelector, CandidateBuildDenial> {
    let previous_identity = selected.identity();
    let current_identity =
        RootSelectorIdentity::new(generation).ok_or(CandidateBuildDenial::Invalid)?;
    let previous = DurableRootSelector::new(
        store,
        format,
        previous_identity,
        RootSelectorRole::Previous,
        selected.root_generation(),
        Some(current_identity),
        Some(generation),
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let current = DurableRootSelector::new(
        store,
        format,
        current_identity,
        RootSelectorRole::Current,
        generation,
        Some(previous_identity),
        Some(selected.root_generation()),
    )
    .ok_or(CandidateBuildDenial::Invalid)?;
    let catalog = BootstrapCatalog::new(
        store,
        format,
        CurrentRootCatalogEntry::new(
            CurrentRootCatalogGeneration::new(generation).ok_or(CandidateBuildDenial::Invalid)?,
        ),
    );
    build.push(
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            publication,
        },
        previous.encode().to_vec(),
    )?;
    build.push(
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            publication,
        },
        current.encode().to_vec(),
    )?;
    build.push(
        RecordArtifactFile::CatalogCandidate { publication },
        catalog.encode().to_vec(),
    )?;
    Ok(current)
}

pub(super) struct CandidateBuild {
    format: PhysicalRecordFormatDeclaration,
    artifacts: Vec<RecoveryPublicationCandidateArtifact>,
}

impl CandidateBuild {
    fn push(
        &mut self,
        artifact: RecordArtifactFile,
        bytes: Vec<u8>,
    ) -> Result<(), CandidateBuildDenial> {
        if bytes.is_empty()
            || self
                .artifacts
                .iter()
                .any(|candidate| candidate.artifact == artifact)
        {
            return Err(CandidateBuildDenial::Invalid);
        }
        self.push_owned(artifact, bytes.into_boxed_slice())
    }

    fn push_owned(
        &mut self,
        artifact: RecordArtifactFile,
        bytes: Box<[u8]>,
    ) -> Result<(), CandidateBuildDenial> {
        if bytes.is_empty()
            || self
                .artifacts
                .iter()
                .any(|candidate| candidate.artifact == artifact)
        {
            return Err(CandidateBuildDenial::Invalid);
        }
        let payload_digest = Sha256::digest(bytes.as_ref()).into();
        self.artifacts.push(RecoveryPublicationCandidateArtifact {
            artifact,
            bytes,
            payload_digest,
        });
        Ok(())
    }
}
