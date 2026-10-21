//! Exact native lineage-population proof for the private adoption inventory.

use super::inventory::WorkflowAdoptionInventoryRequest;
use super::WorkflowAdoptionReadDenial as Denial;

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenial;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::VisibilityProjectionView;

pub(super) fn is_empty(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    view: &VisibilityProjectionView,
    request: &WorkflowAdoptionInventoryRequest<'_>,
) -> Result<bool, Denial> {
    // This is the caller's remaining selection allowance, not a new host
    // allowance. Its exact charge is returned and deducted before truth reads.

    let lineage = &request.layout.lineage;
    // Currency is one logical metadata selection. The native owner pays its
    // real branch-key comparisons/copies and any authentic fork preparation
    // on this same original execution, even when the selected index is warm.

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
    selected
        .population_is_empty(execution)
        .map_err(index_denial)
}

fn index_denial(cause: BoundedEntityFieldLookupDenial) -> Denial {
    match cause.kind() {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => Denial::ReadExecution(cause),
        _ => Denial::SelectedIndexUnavailable,
    }
}
