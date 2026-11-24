//! Recover selected index generations on the original request payer.
use super::{
    basis, owner::IndexMaintenanceOwner, request_payment, root_view::IndexRootView, MaintenanceWork,
};
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::history::data::coordinates::BranchId;
use crate::identity::data::VersionId;
use crate::indexes::data::*;
use crate::mvcc::observation::RelationalBranchObservation;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
type Denial = DerivedIndexMaintenanceDenialKind;

pub(super) fn reconstruct_for_basis(
    owner: &IndexMaintenanceOwner<'_>,
    lane: &ReconstructionLane<'_>,
    execution: &ExecutionRequest<'_, '_>,
    request: &DerivedIndexBuildRequest,
    basis: &super::AdmittedRelationalBranchBasis,
    budget: DerivedIndexMaintenanceBudget,
    mut project: impl FnMut(
        &RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView, RelationalBranchBasisDenial>,
) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
    let (prepared, counts) = request_payment::run(execution, budget, |work| {
        let observation = basis.observation();
        let projection = project(&observation).map_err(Denial::Basis)?;
        basis::validate(request, &observation, None, &projection, execution)?;
        let after = IndexRootView::from_projection(&projection)?;
        require_routing(owner, execution, after.root())?;
        publish(owner, lane, execution, request, &after, work)
    })?;
    Ok(DerivedIndexMaintenanceOutcome {
        generations: prepared.value,
        work: counts,
    })
}

pub(super) fn reconstruct_for_commit(
    owner: &IndexMaintenanceOwner<'_>,
    history: &HistorySubsystem,
    lane: &ReconstructionLane<'_>,
    execution: &ExecutionRequest<'_, '_>,
    request: &DerivedIndexBuildRequest,
    budget: DerivedIndexMaintenanceBudget,
    mut project: impl FnMut(
        &RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView, RelationalBranchBasisDenial>,
    mut project_commit: impl FnMut(
        crate::history::data::coordinates::CommitId,
        BranchId,
        VersionId,
    ) -> Result<VisibilityProjectionView, RelationalBranchBasisDenial>,
) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
    let (prepared, counts) = request_payment::run(execution, budget, |work| {
        work.charge(request.index_ids.len())?;
        let recorded = history
            .commit_artifact(request.source_commit_id)
            .ok_or(Denial::CommitMismatch)?;
        let identity = recorded.identity();
        let branch_bytes = request.branch_id.0.len();
        // Compare the authoring name and copy the registry cell identity.
        work.prepare(2 * branch_bytes as u64 + 2, branch_bytes as u64)?;
        if identity.authoring_branch() != &request.branch_id {
            return Err(Denial::CommitMismatch);
        }
        let projection = match super::retained_projection::select_registered_retained_commit(
            history,
            execution,
            request,
            identity.version_id(),
            work,
        )? {
            Some(observation) => project(&observation),
            None => {
                work.prepare(branch_bytes as u64 + 1, branch_bytes as u64)?;
                project_commit(
                    request.source_commit_id,
                    request.branch_id.clone(),
                    identity.version_id(),
                )
            }
        }
        .map_err(Denial::Basis)?;
        let root = projection.selected_root().ok_or(Denial::CommitMismatch)?;
        work.prepare(branch_bytes as u64 + 1, 0)?;
        if root.commit_receipt().is_none_or(|receipt| {
            receipt.commit_id != request.source_commit_id
                || receipt.version_id != identity.version_id()
        }) || root
            .commit_evidence()
            .is_none_or(|evidence| evidence.authoring_branch() != &request.branch_id)
            || root.descriptor() != Some(recorded.roots())
        {
            return Err(Denial::CommitMismatch);
        }
        require_routing(owner, execution, root)?;
        let after = IndexRootView::from_projection(&projection)?;
        publish(owner, lane, execution, request, &after, work)
    })?;
    Ok(DerivedIndexMaintenanceOutcome {
        generations: prepared.value,
        work: counts,
    })
}

fn require_routing(
    owner: &IndexMaintenanceOwner<'_>,
    execution: &ExecutionRequest<'_, '_>,
    root: &crate::branch::root::RelationalBranchRoot,
) -> Result<(), Denial> {
    let schema = root.schema_authority();
    owner.catalog().routing_for_request(
        execution,
        schema.authority_digest(),
        schema.aspect_plans(),
    )?;
    Ok(())
}

fn publish(
    owner: &IndexMaintenanceOwner<'_>,
    lane: &ReconstructionLane<'_>,
    execution: &ExecutionRequest<'_, '_>,
    request: &DerivedIndexBuildRequest,
    after: &IndexRootView,
    work: &mut MaintenanceWork<'_, '_, '_>,
) -> Result<Vec<DerivedIndexGeneration>, Denial> {
    owner
        .prepare_recovery(lane, execution, request, after, work)?
        .publish_admitted(owner.indexes, work)
}
