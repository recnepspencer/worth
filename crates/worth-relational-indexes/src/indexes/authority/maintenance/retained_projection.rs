//! Explicit reconstruction can project an actually held recovered basis.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::{
    assemble_admitted_relational_branch_basis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::history::retention::lane::RelationalRetentionOwnerRelationship;
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservation;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;

pub(super) fn select_registered_retained_commit(
    history: &HistorySubsystem,
    execution: &ExecutionRequest<'_, '_>,
    request: &DerivedIndexBuildRequest,
    version: VersionId,
    work: &mut MaintenanceWork,
) -> Result<Option<RelationalBranchObservation>, DerivedIndexMaintenanceDenialKind> {
    let Some(cell) = history.branch_cell(&request.branch_id) else {
        return Ok(None);
    };
    // Preserve this door's historical scope. Live-head work uses its exact basis.
    if cell.root().and_then(|root| root.commit_id()) == Some(request.source_commit_id) {
        return Ok(None);
    }
    let entries = cell
        .retained_bases_for_request(execution, |count| work.charge(count))
        .map_err(|cause| match cause {
            worth_relational_branch::internal_port::branch::basis_registry::RelationalRetainedBasisSnapshotStop::Payment(cause) => {
                cause
            }
            worth_relational_branch::internal_port::branch::basis_registry::RelationalRetainedBasisSnapshotStop::Buffer(cause) => {
                basis::buffer_stop(cause)
            }
            worth_relational_branch::internal_port::branch::basis_registry::RelationalRetainedBasisSnapshotStop::Changed => {
                DerivedIndexMaintenanceDenialKind::SnapshotUnavailable
            }
        })?;
    let owner = history.retention_binding();
    for entry in entries.iter() {
        work.prepare(3, 0)?;
        let Some(inner) = entry.upgrade() else {
            continue;
        };
        let actual = assemble_admitted_relational_branch_basis(inner);
        let observation = actual.observation();
        if observation.commit_receipt().is_none_or(|receipt| {
            receipt.commit_id != request.source_commit_id || receipt.version_id != version
        }) {
            continue;
        }
        if actual.owner_inner().retention.owner_relationship(&owner)
            != RelationalRetentionOwnerRelationship::SameOwner
        {
            continue;
        }
        let branch = observation.identity().branch_id();
        work.prepare(branch.0.len().min(request.branch_id.0.len()) as u64 + 1, 0)?;
        if branch != &request.branch_id {
            continue;
        }
        // VisibilitySnapshotBasis owns a copy of this exact selected name.
        work.prepare(branch.0.len() as u64 + 1, branch.0.len() as u64)?;
        return Ok(Some(observation));
    }
    Ok(None)
}
