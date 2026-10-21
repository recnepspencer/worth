use std::sync::Mutex;

mod prefix;
pub(crate) mod records;
mod retained;
pub(crate) use retained::RetainedSettlement;

use worth_foundational::{
    ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture, ExecutionReport,
};

use super::{
    admission::BatchDeclaration,
    meter::RunLimits,
    port::{BatchOutcome, TaskOutcome},
};

/// Completion order is discarded here. Only the canonical identity prefix
/// decides published values, charged work, span, and the reported failure.
pub(crate) fn settle<R, E: crate::ChargedBytes>(
    batch: &BatchDeclaration,
    outcomes: Vec<Mutex<Option<TaskOutcome<R, E>>>>,
    limits: &RunLimits,
    posture: ExecutionPosture,
    active_workers_high_watermark: usize,
    peak_charged_memory_bytes: u64,
    peak_queue_width: usize,
    steals: Option<u64>,
    fallback: Option<ExecutionFallbackCause>,
    mut retained: Option<&mut RetainedSettlement<'_, R>>,
    lease: Option<&crate::ExecutionResourceLease<'_>>,
) -> BatchOutcome<R, E> {
    let mut prefix = prefix::Prefix::new(batch.len());
    for (index, slot) in outcomes.into_iter().enumerate() {
        if let Some(retained) = retained.as_mut() {
            retained.before(Some(batch.identities()[index]), &mut prefix, lease, limits);
        }
        let outcome = slot
            .into_inner()
            .unwrap_or_else(|error| error.into_inner())
            .expect("every worker joined before settlement");
        if let Some(value) = prefix.accept(batch.identities()[index], outcome, limits) {
            prefix.values.push(value);
        }
    }
    if let Some(retained) = retained {
        retained.before(None, &mut prefix, lease, limits);
    }
    limits.add_discarded_work(prefix.discarded);
    let report = ExecutionReport::new(
        posture,
        prefix.work,
        prefix.span,
        ExecutionPhysicalReport::new(
            active_workers_high_watermark,
            peak_charged_memory_bytes,
            steals,
            peak_queue_width,
            limits.discarded_work(),
        ),
    );
    BatchOutcome {
        values: prefix.values,
        stop: prefix.stop,
        prefix_boundary: prefix.boundary,
        report: fallback.map_or(report, |cause| report.with_fallback(cause)),
        stop_owner: prefix.owner,
    }
}
