//! Qualification work: ledger stops and the charged scheduling share.
use super::LedgerStop;
use super::*;

pub(super) fn ledger_stop(subject: &'static str, LedgerStop(cause): LedgerStop) -> CoupledSourceReadStop {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    use worth_execution::ExecutionOrderedMapDenial as Map;
    CoupledSourceReadStop::capture(match cause {
        Map::Allocation(cause) => Attempt::allocation_denied(subject, cause),
        Map::Request(cause) => Attempt::relational_read_stopped(cause.into(), subject),
        Map::CapacityOverflow => Attempt::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            subject,
        ),
    })
}

/// Charge qualification work; it is the wave's scheduling share, observed so.
pub(super) fn charge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &'static str,
    work: u64,
) -> Result<(), CoupledSourceReadStop> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    synchronous_outputs::observe_scheduling_work(work);
    let denied = |cause: worth_execution::WorkCeilingDenial| {
        CoupledSourceReadStop::capture(Attempt::relational_read_stopped(cause.into(), subject))
    };
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(denied)?
        .0
        .map_err(|cause| denied(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
