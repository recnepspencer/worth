use std::sync::Mutex;

use worth_foundational::{
    ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture, ExecutionReport,
};

use super::{
    admission::BatchDeclaration,
    meter::{KernelFailure, KernelStop, NestedStopOwner, RunLimits},
    port::{BatchOutcome, BatchStop, TaskOutcome},
};

/// Completion order is discarded here. Only the canonical identity prefix
/// decides published values, charged work, span, and the reported failure.
pub(crate) fn settle<R, E>(
    batch: &BatchDeclaration,
    outcomes: Vec<Mutex<Option<TaskOutcome<R, E>>>>,
    limits: &RunLimits,
    posture: ExecutionPosture,
    active_workers_high_watermark: usize,
    peak_charged_memory_bytes: u64,
    peak_queue_width: usize,
    steals: Option<u64>,
    fallback: Option<ExecutionFallbackCause>,
) -> BatchOutcome<R, E> {
    let mut values = Vec::with_capacity(batch.len());
    let mut charged_work = 0_u64;
    let mut charged_span = 0_u64;
    let mut discarded_work = 0_u64;
    let mut stop = None;
    let mut prefix_boundary = None;
    let mut stop_owner = None;
    for (index, slot) in outcomes.into_iter().enumerate() {
        let outcome = slot
            .into_inner()
            .unwrap_or_else(|error| error.into_inner())
            .expect("every worker joined before settlement");
        // A discarded suffix still contributed its request stop. Its value
        // and work remain outside the canonical prefix, but its owner cannot
        // become computational merely because an earlier member failed.
        let (result, owner) = outcome.result.into_parts(limits);
        if owner == Some(NestedStopOwner::Request) {
            stop_owner = owner;
        }
        if stop.is_some() {
            discarded_work = discarded_work.saturating_add(outcome.work);
            continue;
        }
        let identity = batch.identities()[index];
        let next = charged_work.checked_add(outcome.work);
        if next.is_none_or(|sum| limits.ceiling().is_some_and(|ceiling| sum > ceiling)) {
            if stop_owner != Some(NestedStopOwner::Request) {
                stop_owner = Some(if next.is_none() {
                    NestedStopOwner::Request
                } else {
                    limits.work_stop_owner()
                });
            }
            stop = Some(if next.is_none() {
                BatchStop::Failure {
                    identity,
                    cause: KernelFailure::Stop(KernelStop::WorkCounterOverflow),
                }
            } else {
                BatchStop::WorkExhausted { identity }
            });
            prefix_boundary = Some(identity);
            discarded_work = discarded_work.saturating_add(outcome.work);
            continue;
        }
        charged_work += outcome.work;
        charged_span = charged_span.max(outcome.span);
        match result {
            Ok(value) => values.push(value),
            Err(KernelFailure::Stop(KernelStop::WorkCeiling)) => {
                if stop_owner != Some(NestedStopOwner::Request) {
                    stop_owner = owner;
                }
                stop = Some(BatchStop::WorkExhausted { identity });
                prefix_boundary = Some(identity);
            }
            Err(cause) => {
                if stop_owner != Some(NestedStopOwner::Request) {
                    stop_owner = owner;
                }
                stop = Some(BatchStop::Failure { identity, cause });
                prefix_boundary = Some(identity);
            }
        }
    }
    limits.add_discarded_work(discarded_work);
    let report = ExecutionReport::new(
        posture,
        charged_work,
        charged_span,
        ExecutionPhysicalReport::new(
            active_workers_high_watermark,
            peak_charged_memory_bytes,
            steals,
            peak_queue_width,
            limits.discarded_work(),
        ),
    );
    BatchOutcome {
        values,
        stop,
        prefix_boundary,
        report: fallback.map_or(report, |cause| report.with_fallback(cause)),
        stop_owner,
    }
}
