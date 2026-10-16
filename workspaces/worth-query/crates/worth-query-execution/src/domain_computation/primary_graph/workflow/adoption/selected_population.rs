//! Exact native lineage-population proof for the private adoption inventory.

use super::inventory::WorkflowAdoptionInventoryRequest;
use super::WorkflowAdoptionReadDenial as Denial;
use crate::domain_computation::primary_graph::{
    index_currency::selected_index_is_current_admitted,
    output_lineage::invalidation::InvalidationEditAdmission,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    indexes::{
        BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupDenial,
        BoundedEntityFieldLookupDenialKind, DerivedIndexBuildRequest,
        DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceDenialKind,
    },
    mvcc::{CompanionPreflightBudget, CompanionPreflightStop},
    runtime::{RelationalRuntime, VisibilityProjectionView},
};

pub(super) fn is_empty(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    view: &VisibilityProjectionView,
    request: &WorkflowAdoptionInventoryRequest<'_>,
) -> Result<(bool, usize), Denial> {
    // This is the caller's remaining selection allowance, not a new host
    // allowance. Its exact charge is returned and deducted before truth reads.
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: request.maximum_work_units as u64,
        maximum_preparation_bytes: request.maximum_preparation_bytes,
    });
    let lineage = &request.layout.lineage;
    if !selected_index_is_current_admitted(
        runtime,
        basis,
        lineage.identity_index_id,
        &mut admission,
    )
    .map_err(Denial::Preparation)?
    {
        let observation = basis.observation();
        let source_commit_id = observation
            .commit_id()
            .ok_or(Denial::SelectedIndexUnavailable)?;
        let branch = observation.identity().branch_id();
        let branch_bytes = u64::try_from(branch.0.len()).map_err(|_| {
            Denial::Preparation(CompanionPreflightStop::PreparationMemoryCounterOverflow)
        })?;
        let bytes = branch_bytes
            .checked_add(std::mem::size_of_val(&lineage.identity_index_id) as u64)
            .ok_or(Denial::Preparation(
                CompanionPreflightStop::PreparationMemoryCounterOverflow,
            ))?;
        let work = branch_bytes.checked_add(2).ok_or(Denial::Preparation(
            CompanionPreflightStop::WorkCounterOverflow,
        ))?;
        admission
            .admit_read_scratch(bytes)
            .and_then(|()| admission.charge_external_work(work))
            .map_err(Denial::Preparation)?;
        // Only this selected field index enters cold reconstruction. Native
        // work and storage spend the SAME remaining caller admission below.
        let built = runtime.index_authority().refresh_field_indexes_for_basis_admitted(
            DerivedIndexBuildRequest {
                source_commit_id, branch_id: branch.clone(),
                index_ids: vec![lineage.identity_index_id],
            }, basis,
            crate::domain_computation::primary_graph::index_maintenance_budget::cold_index_reconstruction_budget(),
            execution,
            |work, bytes| {
                admission.admit_read_scratch(bytes)?;
                admission.charge_external_work(work)
            },
        ).map_err(|stop| match stop {
            DerivedIndexMaintenanceAdmissionStop::Admission(cause) => Denial::Preparation(cause),
            DerivedIndexMaintenanceAdmissionStop::Native(cause) => match cause.kind {
                DerivedIndexMaintenanceDenialKind::Execution(cause) => Denial::ReadExecution(cause),
                DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded => Denial::WorkLimitExceeded {
                    consumed_work_units: cause.work.work_units,
                },
                _ => Denial::SelectedIndexUnavailable,
            },
        })?;
        if built.generations.len() != 1 {
            return Err(Denial::SelectedIndexUnavailable);
        }
    }
    admission
        .charge_external_work(1)
        .map_err(Denial::Preparation)?;
    let selected = runtime
        .index_access()
        .prepare_entity_field_lookup(
            view,
            lineage.identity_index_id,
            lineage.entity_kind,
            &lineage.identity,
            execution,
        )
        .map_err(index_denial)?;
    let empty = selected
        .population_is_empty_with_admission(execution, || admission.charge_external_work(1))
        .map_err(|stop| match stop {
            BoundedEntityFieldLookupAdmissionStop::Admission(cause) => Denial::Preparation(cause),
            BoundedEntityFieldLookupAdmissionStop::Lookup(cause) => index_denial(cause),
            BoundedEntityFieldLookupAdmissionStop::AccountingOverflow
            | BoundedEntityFieldLookupAdmissionStop::ExactBasisRequired => {
                Denial::SelectedIndexUnavailable
            }
        })?;
    Ok((
        empty,
        request.maximum_work_units - admission.remaining_work(),
    ))
}

fn index_denial(cause: BoundedEntityFieldLookupDenial) -> Denial {
    match cause.kind() {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => Denial::ReadExecution(cause),
        _ => Denial::SelectedIndexUnavailable,
    }
}
