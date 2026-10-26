//! Pin immutable native custody at the runtime borrowing boundary.
use super::*;
pub(super) fn pin_runtime_read(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: &read_plan::ReadPlan<'_>,
    additional_indexes: &[worth_relational::facade::indexes::DerivedIndexId],
    additional_denial: WorthQueryApplicationReadExecutionDenialKind,
) -> Result<
    worth_relational::facade::runtime::RelationalPinnedRead,
    WorthQueryApplicationReadExecutionDenial,
> {
    runtime
        .read_truth()
        .pin_read_capability(plan.snapshot, additional_indexes, execution)
        .map_err(|denial| {
            use worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial as Pin;
            match denial {
                Pin::Request(stop) => denial::relational_read_denial(stop, plan.name),
                Pin::IndexNotInstalled(_) => read_execution_denial(additional_denial, plan.name),
                Pin::SnapshotUnavailable => plan.snapshot_unavailable_denial(),
            }
        })
}
