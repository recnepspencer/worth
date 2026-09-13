use std::collections::{BTreeSet, VecDeque};

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile,
};

use super::artifact_read::{required_source, retain_successor};
use super::denial::{admit_successor_read, consume_successor, invalid};
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
) -> Result<Vec<CurrentPhysicalRecordPlacement>, PhysicalRecoverySuccessorCandidateDenial> {
    let mut pending = root.routing_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::RootRoutingBlock {
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
            discovery.read_root_routing_block(
                reference.generation(),
                reference.block(),
                byte_limit,
            ),
            artifact,
        )?;
        let tree =
            PhysicalTreeIdentity::new(root.tree_identity()).ok_or_else(|| invalid(artifact))?;
        let projected = crate::integrity_ingress::projection::root_routing_block(
            &source,
            discovery.store_identity(),
            format,
            tree,
            reference,
            root.node_capacity(),
            integrity_trace,
        )
        .map_err(
            |rejection| PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                artifact,
                generation: reference.generation(),
                denial: rejection.diagnostic(),
            },
        )?;
        let block = projected.block;
        if let Some(found) = block.entries() {
            consume_successor(budget, found.len(), artifact)?;
            materialization.retain_placements(found.len());
            entries.extend_from_slice(found);
        } else if let Some(children) = block.children() {
            consume_successor(budget, children.len(), artifact)?;
            pending.extend(children.iter().copied());
        }
        let bytes = source
            .into_bytes()
            .expect("source-bound root-routing admission retained present bytes");
        let retained_bytes = bytes.len();
        if retain_successor(root.generation(), artifact, bytes, artifacts) {
            materialization.retain_artifact(retained_bytes);
        }
    }
    entries.sort_unstable_by_key(|entry| entry.record());
    Ok(entries)
}
