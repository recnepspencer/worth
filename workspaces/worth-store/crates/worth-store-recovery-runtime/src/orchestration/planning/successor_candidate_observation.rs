use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::manifest_entry_budget::ManifestEntryBudget;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::progression::RecoveryObservedSuccessorCandidate;

mod artifact_read;
mod attempt;
mod denial;
mod free_space;
mod materialization;
mod root_manifest;
mod root_routing;
mod segment_membership;

use artifact_read::observed;
pub(super) use attempt::observe;
use denial::invalid;
use materialization::CandidateMaterialization;

fn observe_bounded(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    materialization: &mut CandidateMaterialization,
    root_protocol_counters: &mut crate::entry::PhysicalRecoveryRootProtocolCounters,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<Option<RecoveryObservedSuccessorCandidate>, PhysicalRecoverySuccessorCandidateDenial> {
    let Some(observed_root) = root_manifest::read(
        discovery,
        selected,
        format,
        byte_limit,
        materialization,
        root_protocol_counters,
    )?
    else {
        return Ok(None);
    };
    let root = observed_root.manifest;
    let root_artifact = observed_root.artifact;
    let mut artifacts = vec![observed(root_artifact, observed_root.bytes)];
    let mut referenced_artifacts = vec![root_artifact];
    let placements = root_routing::read(
        discovery,
        &root,
        format,
        budget,
        byte_limit,
        &mut artifacts,
        &mut referenced_artifacts,
        materialization,
        integrity_trace,
    )?;
    let segment_entries = segment_membership::read(
        discovery,
        &root,
        format,
        budget,
        byte_limit,
        &mut artifacts,
        &mut referenced_artifacts,
        materialization,
        integrity_trace,
    )?;
    let (free_space, free_entries) = free_space::read(
        discovery,
        &root,
        format,
        budget,
        byte_limit,
        &mut artifacts,
        &mut referenced_artifacts,
        materialization,
        integrity_trace,
    )?;
    artifacts.sort_unstable_by_key(|item| item.artifact);
    if artifacts
        .windows(2)
        .any(|pair| pair[0].artifact == pair[1].artifact)
    {
        return Err(invalid(root_artifact));
    }
    referenced_artifacts.sort_unstable();
    if referenced_artifacts
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(invalid(root_artifact));
    }
    Ok(Some(RecoveryObservedSuccessorCandidate {
        root,
        free_space,
        placements: placements.into_boxed_slice(),
        segment_entries: segment_entries.into_boxed_slice(),
        free_entries: free_entries.into_boxed_slice(),
        referenced_artifacts: referenced_artifacts.into_boxed_slice(),
        artifacts: artifacts.into_boxed_slice(),
    }))
}

pub(super) const fn artifact_generation(artifact: RecordArtifactFile) -> u64 {
    match artifact {
        RecordArtifactFile::RootManifest { generation }
        | RecordArtifactFile::RootRoutingBlock { generation, .. }
        | RecordArtifactFile::SegmentMembershipBlock { generation, .. }
        | RecordArtifactFile::FreeSpaceManifest { generation }
        | RecordArtifactFile::FreeSpaceMembershipBlock { generation, .. } => generation,
        _ => 0,
    }
}
