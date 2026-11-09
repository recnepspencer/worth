//! Exact native lineage-population proof for the private adoption inventory.

use super::WorkflowAdoptionReadDenial as Denial;
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

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
    layout: &WorthQueryWorkflowLayout,
) -> Result<bool, Denial> {
    // Native index admission uses this exact execution and source. Bounded
    // graph-record reads remain the inventory owner's separate responsibility.
    let lineage = &layout.lineage;
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

/// An explicit Home transition reads its exact authoritative source. It must not
/// require or rebuild a live-adoption derived generation during cold planning.
pub(in crate::domain_computation::primary_graph) fn workflow_record_population_is_empty(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    layout: &WorthQueryWorkflowLayout,
    maximum_work_units: usize,
) -> Result<(bool, usize), Denial> {
    let read = view
        .bounded_entities_of_kind(execution, layout.lineage.entity_kind, maximum_work_units)
        .map_err(Denial::ReadExecution)?
        .map_err(|limit| Denial::WorkLimitExceeded {
            consumed_work_units: limit.consumed_work_units(),
        })?;
    Ok((read.records().is_empty(), read.work_units()))
}
