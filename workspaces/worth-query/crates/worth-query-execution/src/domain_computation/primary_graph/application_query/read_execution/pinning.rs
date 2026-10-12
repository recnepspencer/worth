//! Pin immutable native custody at the runtime borrowing boundary.
use super::*;
pub(super) fn pin_runtime_read(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &read_plan::ReadPlan<'_>,
    additional_indexes: &[worth_relational::facade::indexes::DerivedIndexId],
    additional_denial: WorthQueryApplicationReadExecutionDenialKind,
) -> Result<
    worth_relational::facade::runtime::RelationalPinnedRead,
    WorthQueryApplicationReadExecutionDenial,
> {
    let mut selected = plan.selected_indexes(graph);
    let predicate_count = selected.len();
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
                Pin::IndexNotInstalled(index) => {
                    if selected[..predicate_count].contains(&index) {
                        plan.predicate_unavailable_denial()
                    } else {
                        read_execution_denial(additional_denial, plan.name)
                    }
                }
                Pin::SnapshotUnavailable => plan.snapshot_unavailable_denial(),
            }
        })
}
