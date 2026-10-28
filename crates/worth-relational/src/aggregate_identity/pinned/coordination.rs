//! One exact-source, original-funded handoff for Native aggregate read audiences.
use super::super::funding::AggregateWork;
use super::*;
use crate::branch::RelationalBranchRoot;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};

#[derive(Clone, Copy, Default)]
pub(super) struct Limits<'input> {
    pub(super) maximum_work: Option<u64>,
    pub(super) control: Option<&'input dyn Fn() -> Result<(), Cause>>,
}

pub(super) fn read<Snapshot, Value: ChargedBytes, Output>(
    pin: &RelationalPinnedRead<Snapshot>,
    execution: &ExecutionRequest<'_, '_>,
    limits: Limits<'_>,
    build: impl FnOnce(
        &RelationalBranchRoot,
        &InvariantStateView<'_>,
        &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<Value>,
    finish: impl FnOnce(
        Value,
        AggregateReadObservations,
        u64,
        Option<ExecutionMemoryReservation>,
    ) -> Output,
) -> Result<Output, PinnedAggregateReadDenial> {
    let mut reads = AggregateReadObservations::default();
    let mut work = 0u64;
    let mut settled = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        settled = Some((|| {
            context
                .poll()
                .map_err(|stop| FundingFailure::Execution(stop.into()))?;
            let root =
                RelationalBranchRootState::new(std::sync::Arc::clone(pin.pinned_basis().root()));
            let view = InvariantStateView::new(&root, pin.pinned_basis().version_id());
            let mut checkpoint = |units| read_checkpoint(limits, &mut work, units);
            let (value, retention) =
                funding::run_read(execution, &mut checkpoint, &mut reads, |funded| {
                    build(root.root(), &view, funded)
                })?;
            let result = finish(value, reads, work, retention);
            if let Some(control) = limits.control {
                control().map_err(FundingFailure::Execution)?;
            }
            context
                .poll()
                .map_err(|stop| FundingFailure::Execution(stop.into()))?;
            Ok(result)
        })());
    });
    let result = match (handoff, settled) {
        (Ok(((), _)), Some(result)) => result,
        (
            Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )),
            Some(Err(failure)),
        ) => Err(failure),
        (Err(stop), _) => Err(FundingFailure::Execution(stop.into())),
        (Ok(_), None) => unreachable!("successful Native entry invokes its read"),
    };
    result.map_err(|failure| PinnedAggregateReadDenial {
        kind: denial::kind(failure),
        reads,
        work,
    })
}

fn read_checkpoint(limits: Limits<'_>, work: &mut u64, units: u64) -> BuildResult<()> {
    *work = work
        .checked_add(units)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    if let Some(control) = limits.control {
        control().map_err(|cause| MapKernelFailure::Domain(PlanDenial::Execution(cause)))?;
    }
    if limits.maximum_work.is_some_and(|maximum| *work > maximum) {
        return Err(MapKernelFailure::Domain(PlanDenial::ReadWorkLimitExceeded));
    }
    Ok(())
}
