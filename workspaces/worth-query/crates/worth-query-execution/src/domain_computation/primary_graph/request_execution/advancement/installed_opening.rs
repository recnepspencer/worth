//! An installed World lends one branded request for each public call.
use super::source::WorthQueryAdvancementSource;
use super::*;

/// Opens one host advancement on `placement` for `source`, under `cancellation` and
/// `deadline`. Custody follows the owner into descendant workers; unrelated calls own theirs.
pub(crate) fn open_placed_advancement<R>(
    placement: worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'_>,
    source: &(impl WorthQueryAdvancementSource + ?Sized),
    cancellation: worth_execution::CancellationToken,
    deadline: Option<worth_execution::MonotonicDeadline>,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    QueryRequestExecution::open_control(placement, cancellation, deadline).run_advancement(
        source.advancement_owner(),
        Some(source.advancement_source_instance()),
        body,
    )
}

/// Opens one host advancement whose budget is a controlled child of the caller's `lease`,
/// so every worker descends from it and the caller's charges stay inside one budget.
pub(crate) fn open_leased_advancement<R>(
    lease: &worth_execution::ExecutionResourceLease<'_>,
    source: &(impl WorthQueryAdvancementSource + ?Sized),
    cancellation: worth_execution::CancellationToken,
    deadline: Option<worth_execution::MonotonicDeadline>,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let _custody = OpeningCustody::enter()?;
    QueryRequestExecution::under_lease(lease, cancellation, deadline).run_advancement(
        source.advancement_owner(),
        Some(source.advancement_source_instance()),
        body,
    )
}
