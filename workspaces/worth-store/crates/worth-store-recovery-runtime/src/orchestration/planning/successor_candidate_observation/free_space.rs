use std::collections::{BTreeSet, VecDeque};

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

use super::artifact_read::{observed, required_source, retain_successor};
use super::denial::{admit_successor_read, consume_successor, invalid, membership_failure};
use super::materialization::CandidateMaterialization;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::progression::RecoveryObservedCandidateArtifact;

pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    artifacts: &mut Vec<RecoveryObservedCandidateArtifact>,
    referenced_artifacts: &mut Vec<RecordArtifactFile>,
    materialization: &mut CandidateMaterialization,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<
    (
        DurableFreeSpaceManifestHeader,
        Vec<RecordFreeSpaceManifestEntry>,
    ),
    PhysicalRecoverySuccessorCandidateDenial,
> {
    let header_artifact = RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    };
    let header_source = required_source(
        discovery.read_free_space_manifest(root.generation(), byte_limit),
        header_artifact,
    )?;
    let header = crate::integrity_ingress::projection::free_space_header(
        &header_source,
        discovery.store_identity(),
        format,
        root,
        integrity_trace,
    )
    .map_err(
        |rejection| PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
            artifact: header_artifact,
            generation: root.generation(),
            denial: rejection.diagnostic(),
        },
    )?;
    let header_bytes = header_source
        .into_bytes()
        .expect("source-bound free-space-header admission retained present bytes");
    materialization.retain_free_space_header(header_bytes.len());
    artifacts.push(observed(header_artifact, header_bytes));
    referenced_artifacts.push(header_artifact);
    materialization.retain_reference();
    let mut pending = header.root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        referenced_artifacts.push(artifact);
        materialization.retain_reference();
        admit_successor_read(budget, artifact)?;
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact));
        }
        let source = required_source(
            discovery.read_free_space_membership_block(
                reference.generation(),
                reference.block(),
                byte_limit,
            ),
            artifact,
        )?;
        let tree =
            PhysicalTreeIdentity::new(header.tree_identity()).ok_or_else(|| invalid(artifact))?;
        let block = crate::integrity_ingress::projection::free_space_membership_block(
            &source,
            discovery.store_identity(),
            format,
            tree,
            reference,
            header.node_capacity(),
            budget.remaining(),
            integrity_trace,
        )
        .map_err(|failure| membership_failure(budget, artifact, failure))?;
        if let Some(found) = block.entries() {
            consume_successor(budget, found.len(), artifact)?;
            materialization.retain_free_entries(found.len());
            entries.extend_from_slice(found);
        } else if let Some(children) = block.children() {
            consume_successor(budget, children.len(), artifact)?;
            pending.extend(children.iter().copied());
        }
        let bytes = source
            .into_bytes()
            .expect("source-bound free-space-membership admission retained present bytes");
        let retained_bytes = bytes.len();
        if retain_successor(root.generation(), artifact, bytes, artifacts) {
            materialization.retain_artifact(retained_bytes);
        }
    }
    entries.sort_unstable_by_key(|entry| (entry.class() as u8, entry.owner()));
    Ok((header, entries))
}
