//! Pin immutable native custody at the runtime borrowing boundary.
use super::*;
pub(super) fn pin_runtime_read(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &read_plan::ReadPlan<'_>,
    additional_indexes: &[worth_relational::facade::indexes::DerivedIndexId],
) -> Result<
    worth_relational::facade::runtime::RelationalPinnedRead,
    WorthQueryApplicationReadExecutionDenial,
> {
    let mut selected = plan.selected_indexes(graph);
    selected.extend_from_slice(additional_indexes);
    runtime
        .read_truth()
        .pin_read_capability(plan.snapshot, &selected, execution)
        .map_err(|denial| {
            use worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial as Pin;
            match denial {
                Pin::Request(stop) => denial::relational_cause_denial(
                    worth_relational::facade::transactions::RelationalExecutionDenialCause::from(
                        stop,
                    ),
                    plan.name,
                ),
                Pin::IndexNotInstalled(_) => read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
                    plan.name,
                ),
                Pin::SnapshotUnavailable => read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                    plan.name,
                ),
            }
        })
}
