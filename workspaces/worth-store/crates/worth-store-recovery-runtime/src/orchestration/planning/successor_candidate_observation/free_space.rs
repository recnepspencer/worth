use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

use super::artifact_read::{observed, read as read_artifact, retain_successor};
use super::denial::{consume_successor, invalid, membership_failure};
use super::materialization::CandidateMaterialization;
use super::resident::{memory_failure, trace_slots};
use super::tree_walk_resident::{free_projection_scratch, VisitedNodes};
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::orchestration::planning::manifest_entry_budget::{
    pays_for, spend, ChargeToken, EntryAdmission, ManifestEntryBudget,
};
use crate::progression::{PlanningResidentAllowance, RecoveryObservedCandidateArtifact};

pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    charge: ChargeToken,
    budget: &mut ManifestEntryBudget,
    artifacts: &mut Vec<RecoveryObservedCandidateArtifact>,
    referenced_artifacts: &mut Vec<RecordArtifactFile>,
    materialization: &mut CandidateMaterialization,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    allowance: &mut PlanningResidentAllowance,
) -> Result<
    (
        DurableFreeSpaceManifestHeader,
        Vec<RecordFreeSpaceManifestEntry>,
    ),
    PhysicalRecoverySuccessorCandidateDenial,
> {
    // The free-space tree is the candidate's last read: it spends the charge.
    pays_for(&charge, root.generation());
    let header_artifact = RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    };
    trace_slots(header_artifact, integrity_trace, allowance)?;
    let header_source = read_artifact(discovery, header_artifact, format, &charge, allowance)?;
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
    allowance
        .grow(artifacts, 1)
        .map_err(|failure| memory_failure(header_artifact, failure))?;
    artifacts.push(observed(header_artifact, header_bytes, allowance)?);
    allowance
        .grow(referenced_artifacts, 1)
        .map_err(|failure| memory_failure(header_artifact, failure))?;
    referenced_artifacts.push(header_artifact);
    materialization.retain_reference();
    let mut pending = Vec::new();
    if let Some(reference) = header.root() {
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
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
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
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
            free_projection_scratch(format).map_err(|failure| memory_failure(artifact, failure))?;
        allowance
            .retain(scratch)
            .map_err(|failure| memory_failure(artifact, failure))?;
        trace_slots(artifact, integrity_trace, allowance)?;
        let source = read_artifact(discovery, artifact, format, &charge, allowance)?;
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
        drop(block);
        allowance.release(scratch);
        let bytes = source
            .into_bytes()
            .expect("source-bound free-space-membership admission retained present bytes");
        let retained_bytes = bytes.len();
        if retain_successor(root.generation(), artifact, bytes, artifacts, allowance)? {
            materialization.retain_artifact(retained_bytes);
        }
    }
    let pending_bytes = PlanningResidentAllowance::vector_bytes(&pending)
        .map_err(|failure| memory_failure(header_artifact, failure))?;
    drop(pending);
    allowance.release(pending_bytes);
    visited
        .release(allowance)
        .map_err(|failure| memory_failure(header_artifact, failure))?;
    entries.sort_unstable_by_key(|entry| worth_store_physical_format::FreeSpaceKey::from(*entry));
    spend(charge, root.generation());
    Ok((header, entries))
}
