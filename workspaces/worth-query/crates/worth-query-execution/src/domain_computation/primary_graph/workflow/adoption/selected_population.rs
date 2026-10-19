//! Exact native lineage-population proof for the private adoption inventory.

use super::inventory::WorkflowAdoptionInventoryRequest;
use super::WorkflowAdoptionReadDenial as Denial;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    indexes::{
        BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupDenial,
        BoundedEntityFieldLookupDenialKind, DerivedIndexMaintenanceDenialKind,
    },
    mvcc::CompanionPreflightBudget,
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
        maximum_work_visits: Some(request.maximum_work_units as u64),
        maximum_preparation_bytes: Some(request.maximum_preparation_bytes),
    });
    let lineage = &request.layout.lineage;
    // Currency is one logical metadata selection. The native owner pays its
    // real branch-key comparisons/copies and any authentic fork preparation
    // on this same original execution, even when the selected index is warm.
    admission
        .charge_external_work(1)
        .map_err(Denial::Preparation)?;
    runtime.index_authority().ensure_field_index_for_basis(
        lineage.identity_index_id,
        basis,
        crate::domain_computation::primary_graph::index_maintenance_budget::cold_index_reconstruction_budget(),
        execution,
    ).map_err(|cause| match cause.kind {
        DerivedIndexMaintenanceDenialKind::Execution(cause) => Denial::ReadExecution(cause),
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded => Denial::IndexMaintenanceWorkLimit,
        _ => Denial::SelectedIndexUnavailable,
    })?;
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
        usize::try_from(admission.charged_work()).expect("selected work fits its native count"),
    ))
}

fn index_denial(cause: BoundedEntityFieldLookupDenial) -> Denial {
    match cause.kind() {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => Denial::ReadExecution(cause),
        _ => Denial::SelectedIndexUnavailable,
    }
}
