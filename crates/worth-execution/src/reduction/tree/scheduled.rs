use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::{
        run_checked_batch_with_charge, AdmittedBatch, BackendKind, BatchDenial, BatchStop,
        KernelContext, KernelFailure, KernelStop,
    },
    oracle::CanonicalBits,
    report::ChargedBytes,
};

use super::{
    super::plan::ReductionDenial, staged::FrontierCharge, ReductionMetrics, ReductionPlan,
    ReductionRunFailure, ReductionRunStop, ReductionTree,
};

pub(crate) enum ScheduledReductionError {
    Admission(LeaseDenial),
    Reduction(ReductionRunFailure<KernelStop>),
}

impl<T, F> ReductionTree<T, F>
where
    T: Send + Sync + Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T + Sync,
{
    /// The backend executes independent subtrees speculatively. Their results
    /// are settled with deferred joins in canonical DFS order, so a later
    /// task failure cannot overtake an earlier join failure.
    pub(crate) fn build_scheduled_checked(
        lease: &ExecutionResourceLease<'_>,
        backend: BackendKind,
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        max_value_bytes: u64,
        context: &mut KernelContext<'_, '_>,
    ) -> Result<(Self, ReductionMetrics), ScheduledReductionError> {
        let (mut shape, shape_metrics) =
            Self::prepare_shape_checked(plan, values, &identity, &combine, max_value_bytes, || {
                context.checkpoint(1)
            })
            .map_err(ScheduledReductionError::Reduction)?;
        let frontier = shape.frontier(lease.policy().budget().max_workers().get());
        let task_scratch =
            max_value_bytes
                .checked_mul(6)
                .ok_or(ScheduledReductionError::Admission(
                    LeaseDenial::ChargedBytesOverflow,
                ))?;
        let entries = frontier
            .tasks
            .iter()
            .copied()
            .map(|spec| {
                let capacity = Self::checked_subtree_result_bound(spec.len(), max_value_bytes)
                    .ok_or(ScheduledReductionError::Admission(
                        LeaseDenial::ChargedBytesOverflow,
                    ))?;
                Ok((spec.identity, spec, task_scratch, capacity))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let batch = AdmittedBatch::try_admit(entries, 0).map_err(|denial| match denial {
            BatchDenial::Identities => ScheduledReductionError::Reduction(ReductionRunFailure {
                reason: ReductionRunStop::Denial(ReductionDenial::IdentitiesNotCanonical),
                metrics: ReductionMetrics::default(),
            }),
            BatchDenial::MemoryOverflow => {
                ScheduledReductionError::Admission(LeaseDenial::ChargedBytesOverflow)
            }
        })?;
        // Suppress the backend's task-order parent charge. Canonical tree
        // settlement below charges only the accepted operation prefix.
        let outcomes = run_checked_batch_with_charge(
            Some(lease),
            &batch,
            backend,
            &|spec, child_context| {
                let before = child_context.cost();
                let result = Self::evaluate_subtree_checked(
                    &shape,
                    *spec,
                    &identity,
                    &combine,
                    max_value_bytes,
                    || child_context.checkpoint(1),
                )
                .map_err(KernelFailure::Domain)?;
                if !child_context.apply_structural_span(
                    before,
                    result.metrics.charged_work,
                    result.metrics.charged_span,
                ) {
                    return Err(KernelFailure::Stop(KernelStop::WorkCounterOverflow));
                }
                Ok(result)
            },
            false,
        );
        let failed = match outcomes.stop {
            Some(BatchStop::Admission(denial)) => {
                return Err(ScheduledReductionError::Admission(denial));
            }
            Some(BatchStop::WorkExhausted { identity }) => {
                let spec = frontier
                    .tasks
                    .iter()
                    .find(|spec| spec.identity == identity)
                    .ok_or_else(|| overflow(shape_metrics))?;
                let target = u64::try_from(spec.len())
                    .ok()
                    .and_then(|len| len.checked_mul(4))
                    .ok_or_else(|| overflow(shape_metrics))?;
                Some((
                    identity,
                    stopped(ReductionRunStop::Hook(KernelStop::WorkCeiling)),
                    target,
                ))
            }
            Some(BatchStop::Failure { identity, cause }) => {
                let failure = task_failure(cause);
                let target = failure.metrics.charged_work;
                Some((identity, failure, target))
            }
            None => None,
        };
        let before_evaluation = context.cost();
        let mut accepted_task_work = 0_u64;
        let settled = Self::settle_frontier_checked(
            shape,
            identity,
            combine,
            shape_metrics,
            frontier,
            outcomes.values,
            failed,
            max_value_bytes,
            |charge| match charge {
                FrontierCharge::Node => context.checkpoint(1),
                FrontierCharge::TaskUnit => {
                    context.checkpoint(1)?;
                    accepted_task_work = accepted_task_work
                        .checked_add(1)
                        .ok_or(KernelStop::WorkCounterOverflow)?;
                    Ok(())
                }
            },
        );
        if !context.reconcile_discarded_work(outcomes.report.charged_work(), accepted_task_work) {
            return Err(overflow(shape_metrics));
        }
        let (tree, metrics) = settled.map_err(|mut failure| {
            // The serial checked oracle reports an interrupted traversal's
            // accepted prefix as sequential span. Completed speculative
            // subtrees may have DAG spans, but they cannot change this
            // failure-prefix convention when frontier width changes.
            failure.metrics.charged_span = failure.metrics.charged_work;
            ScheduledReductionError::Reduction(failure)
        })?;
        if outcomes.report.charged_work() != accepted_task_work {
            return Err(overflow(metrics));
        }
        let evaluation_work = metrics
            .charged_work
            .checked_sub(shape_metrics.charged_work)
            .ok_or_else(|| overflow(metrics))?;
        let evaluation_span = metrics
            .charged_span
            .checked_sub(shape_metrics.charged_span)
            .ok_or_else(|| overflow(metrics))?;
        if !context.apply_structural_span(before_evaluation, evaluation_work, evaluation_span) {
            return Err(overflow(metrics));
        }
        Ok((tree, metrics))
    }
}

fn task_failure(
    cause: KernelFailure<ReductionRunFailure<KernelStop>>,
) -> ReductionRunFailure<KernelStop> {
    match cause {
        KernelFailure::Domain(failure) => failure,
        KernelFailure::Stop(stop) => stopped(ReductionRunStop::Hook(stop)),
        KernelFailure::Panic => stopped(ReductionRunStop::Panic),
        KernelFailure::ResultCapacityExceeded => stopped(ReductionRunStop::ResultCapacityExceeded),
    }
}

fn stopped(reason: ReductionRunStop<KernelStop>) -> ReductionRunFailure<KernelStop> {
    ReductionRunFailure {
        reason,
        metrics: ReductionMetrics::default(),
    }
}

fn overflow(metrics: ReductionMetrics) -> ScheduledReductionError {
    ScheduledReductionError::Reduction(ReductionRunFailure {
        reason: ReductionRunStop::WorkCounterOverflow,
        metrics,
    })
}
