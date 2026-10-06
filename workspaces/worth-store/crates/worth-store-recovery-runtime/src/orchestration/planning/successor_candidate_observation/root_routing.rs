use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile,
};

use super::artifact_read::{read as read_artifact, retain_successor};
use super::denial::{consume_successor, invalid};
use super::materialization::CandidateMaterialization;
use super::resident::{memory_failure, trace_slots};
use super::tree_walk_resident::{root_projection_scratch, VisitedNodes};
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::orchestration::planning::manifest_entry_budget::{
    pays_for, ChargeToken, ManifestEntryBudget,
};
use crate::progression::{PlanningResidentAllowance, RecoveryObservedCandidateArtifact};

pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    charge: &ChargeToken,
    budget: &mut ManifestEntryBudget,
    artifacts: &mut Vec<RecoveryObservedCandidateArtifact>,
    referenced_artifacts: &mut Vec<RecordArtifactFile>,
    materialization: &mut CandidateMaterialization,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<CurrentPhysicalRecordPlacement>, PhysicalRecoverySuccessorCandidateDenial> {
    pays_for(charge, root.generation());
    let mut pending = Vec::new();
    if let Some(reference) = root.routing_root() {
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        allowance
            .grow(&mut pending, 1)
            .map_err(|failure| memory_failure(artifact, failure))?;
        pending.push(reference);
    }
    let mut visited = VisitedNodes::new();
    let mut entries = Vec::new();
    let mut cursor = 0;
    while let Some(&reference) = pending.get(cursor) {
        cursor += 1;
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        allowance
            .grow(referenced_artifacts, 1)
            .map_err(|failure| memory_failure(artifact, failure))?;
        referenced_artifacts.push(artifact);
        materialization.retain_reference();
        if !visited
            .insert((reference.generation(), reference.block()), allowance)
            .map_err(|failure| memory_failure(artifact, failure))?
        {
            return Err(invalid(artifact));
        }
        let scratch =
            root_projection_scratch(format).map_err(|failure| memory_failure(artifact, failure))?;
        allowance
            .retain(scratch)
            .map_err(|failure| memory_failure(artifact, failure))?;
        trace_slots(artifact, integrity_trace, allowance)?;
        let source = read_artifact(discovery, artifact, format, charge, allowance)?;
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
        let block = &projected.block;
        if let Some(found) = block.entries() {
            consume_successor(budget, found.len(), artifact)?;
            materialization.retain_placements(found.len());
            allowance
                .grow(&mut entries, found.len())
                .map_err(|failure| memory_failure(artifact, failure))?;
            entries.extend_from_slice(found);
        } else if let Some(children) = block.children() {
            allowance
                .grow(&mut pending, children.len())
                .map_err(|failure| memory_failure(artifact, failure))?;
            pending.extend(children.iter().copied());
        }
        drop(projected);
        allowance.release(scratch);
        let bytes = source
            .into_bytes()
            .expect("source-bound root-routing admission retained present bytes");
        let retained_bytes = bytes.len();
        if retain_successor(root.generation(), artifact, bytes, artifacts, allowance)? {
            materialization.retain_artifact(retained_bytes);
        }
    }
    let pending_bytes = PlanningResidentAllowance::vector_bytes(&pending).map_err(|failure| {
        memory_failure(
            RecordArtifactFile::RootManifest {
                generation: root.generation(),
            },
            failure,
        )
    })?;
    drop(pending);
    allowance.release(pending_bytes);
    visited.release(allowance).map_err(|failure| {
        memory_failure(
            RecordArtifactFile::RootManifest {
                generation: root.generation(),
            },
            failure,
        )
    })?;
    entries.sort_unstable_by_key(|entry| entry.record());
    Ok(entries)
}
