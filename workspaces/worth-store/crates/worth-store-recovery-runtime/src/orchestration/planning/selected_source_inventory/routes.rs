//! Complete, integrity-witnessed routing traversal for an addressed root.
//! Shared by historical completion and preplanning root-chain admission.

use std::collections::{BTreeSet, VecDeque};

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile,
};

use super::{ManifestEntryBudget, PageObservationFailure, ResidentAllowance};
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;

pub(in crate::orchestration::planning) fn observe_routes_with_budget(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<Vec<CurrentPhysicalRecordPlacement>, PageObservationFailure> {
    let mut resident = ResidentAllowance::new(u64::MAX);
    observe_routes_with_resident_budget(discovery, root, format, budget, trace, &mut resident)
}

pub(in crate::orchestration::planning) fn observe_routes_with_resident_budget(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    resident: &mut ResidentAllowance,
) -> Result<Vec<CurrentPhysicalRecordPlacement>, PageObservationFailure> {
    let tree = PhysicalTreeIdentity::new(root.tree_identity())
        .ok_or(PageObservationFailure::ManifestEntryLimit)?;
    let mut pending = root.routing_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    while let Some(reference) = pending.pop_front() {
        resident.block(format)?;
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(PageObservationFailure::ManifestEntryLimit);
        }
        budget.consume(1)?;
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        resident.trace_slots(trace, 1)?;
        let observed = super::required_source(
            discovery.read_root_routing_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            ),
            None,
        )?;
        let projected = crate::integrity_ingress::projection::root_routing_block(
            &observed,
            discovery.store_identity(),
            format,
            tree,
            reference,
            root.node_capacity(),
            trace,
        )
        .map_err(|denial| PageObservationFailure::Integrity {
            artifact,
            denial: denial.diagnostic(),
        })?;
        if let Some(found) = projected.block.entries() {
            budget.consume(found.len())?;
            resident.entries(
                found.len(),
                std::mem::size_of::<CurrentPhysicalRecordPlacement>(),
            )?;
            entries.extend_from_slice(found);
        } else if let Some(children) = projected.block.children() {
            budget.consume(children.len())?;
            resident.entries(
                children.len(),
                std::mem::size_of::<worth_store_physical_format::ManifestBlockReference>(),
            )?;
            pending.extend(children.iter().copied());
        } else {
            return Err(PageObservationFailure::InvalidManifest {
                target: None,
                artifact,
            });
        }
    }
    entries.sort_unstable_by_key(|route| route.record());
    if entries.len() as u64 != root.record_count()
        || entries
            .windows(2)
            .any(|pair| pair[0].record() == pair[1].record())
    {
        return Err(PageObservationFailure::ManifestEntryLimit);
    }
    Ok(entries)
}
