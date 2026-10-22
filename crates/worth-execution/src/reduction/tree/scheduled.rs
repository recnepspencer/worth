use std::sync::OnceLock;

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::{
        run_checked_batch_with_charge, AdmittedBatch, BackendKind, BatchDenial, BatchStop,
        KernelContext, KernelFailure, KernelStop, StructuralSpanFailure,
    },
    oracle::CanonicalBits,
    report::ChargedBytes,
};

use super::{
    super::plan::ReductionDenial,
    construction::SubtreeSpec,
    staged::FrontierCharge,
    value_storage::{BoundedValues, NativeValues},
    ReductionMetrics, ReductionPlan, ReductionRunFailure, ReductionRunStop, ReductionTree,
};
mod storage;
use storage::ScheduledValues;

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
        Self::build_scheduled_with_storage(
            lease,
            backend,
            plan,
            values,
            identity,
            combine,
            BoundedValues(max_value_bytes),
            context,
        )
    }
}

impl<T, F> ReductionTree<T, F>
where
    T: Send + Sync + Clone + CanonicalBits,
    F: Fn(&T, &T) -> T + Sync,
{
    pub(crate) fn build_scheduled_native(
        lease: &ExecutionResourceLease<'_>,
        backend: BackendKind,
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        context: &mut KernelContext<'_, '_>,
    ) -> Result<(super::NativeReductionTree<T, F>, ReductionMetrics), ScheduledReductionError> {
        Self::build_scheduled_with_storage(
            lease,
            backend,
            plan,
            values,
            identity,
            combine,
            NativeValues { lease: Some(lease) },
            context,
        )
        .map(|(tree, metrics)| (super::NativeReductionTree::from_raw(tree), metrics))
    }

    fn build_scheduled_with_storage<Storage: ScheduledValues<T>>(
        lease: &ExecutionResourceLease<'_>,
        backend: BackendKind,
        plan: ReductionPlan,
        values: Vec<T>,
        identity: T,
        combine: F,
        storage: Storage,
        context: &mut KernelContext<'_, '_>,
    ) -> Result<(Self, ReductionMetrics), ScheduledReductionError> {
        let accounted = storage.has_accounting();
        let (mut shape, shape_metrics) =
            Self::prepare_shape_with_storage(plan, values, &identity, &combine, storage, || {
                if accounted {
                    context.checkpoint(1)
                } else {
                    context.poll()
                }
            })
            .map_err(ScheduledReductionError::Reduction)?;
        let frontier = shape
            .frontier(lease.policy().max_workers().get(), storage)
            .map_err(|reason| {
                ScheduledReductionError::Reduction(ReductionRunFailure {
                    reason,
                    metrics: shape_metrics,
                })
            })?;
        let batch = storage.batch::<F>(&frontier.tasks)?;
        // Suppress the backend's task-order parent charge. Canonical tree
        // settlement below charges only the accepted operation prefix.
        let outcomes = storage.run(lease, &batch, backend, &|task, child_context| {
            let before = child_context.has_accounting().then(|| child_context.cost());
            let evaluated = Self::evaluate_subtree_with_storage(
                &shape,
                task.spec,
                &identity,
                &combine,
                storage,
                || {
                    if accounted {
                        child_context.checkpoint(1)
                    } else {
                        child_context.poll()
                    }
                },
            );
            // A request stop replaces a task's result in the backend. Keep
            // its checked metrics on the admitted input before that boundary.
            let metrics = match &evaluated {
                Ok(result) => result.metrics,
                Err(failure) => failure.metrics,
            };
            if let Some(completed) = &task.completed {
                completed
                    .set(metrics)
                    .expect("a canonical subtree is evaluated once");
            }
            let result = evaluated.map_err(KernelFailure::Domain)?;
            if let Some(before) = before {
                let metrics = result
                    .metrics
                    .accounted_metrics()
                    .expect("accounted child context");
                child_context
                    .apply_structural_span(before, metrics.charged_work, metrics.charged_span)
                    .map_err(|failure| {
                        KernelFailure::Domain(ReductionRunFailure::structural_span_failure(
                            failure,
                            result.metrics,
                        ))
                    })?;
            }
            Ok(result)
        });

        (|| {
            let failed = match outcomes.stop {
                Some(BatchStop::Allocation(cause)) => {
                    return Err(ScheduledReductionError::Reduction(ReductionRunFailure {
                        reason: ReductionRunStop::Denial(ReductionDenial::Allocation(cause)),
                        metrics: shape_metrics,
                    }));
                }
                Some(BatchStop::Admission(denial)) => {
                    return Err(ScheduledReductionError::Admission(denial));
                }
                Some(BatchStop::WorkExhausted { identity }) => {
                    let spec = frontier
                        .tasks
                        .iter()
                        .find(|spec| spec.identity == identity)
                        .ok_or_else(|| metrics_mismatch(shape_metrics))?;
                    let target = u64::try_from(spec.len())
                        .ok()
                        .and_then(|len| len.checked_mul(4))
                        .ok_or_else(|| overflow(shape_metrics))?;
                    Some((
                        identity,
                        stopped(ReductionRunStop::Hook(KernelStop::WorkCeiling)),
                        Some(target),
                    ))
                }
                Some(BatchStop::Failure { identity, cause }) => {
                    let completed = batch
                        .values()
                        .iter()
                        .find(|task| task.spec.identity == identity)
                        .and_then(|task| task.completed.as_ref()?.get())
                        .copied()
                        .unwrap_or_else(|| ReductionMetrics::for_mode(accounted));
                    let failure = task_failure(cause, completed);
                    let target = failure
                        .metrics
                        .accounted_metrics()
                        .map(|metrics| metrics.charged_work);
                    Some((identity, failure, target))
                }
                None => None,
            };
            let before_evaluation = accounted.then(|| context.cost());
            let mut accepted_task_work = accounted.then_some(0_u64);
            let settled = Self::settle_frontier_with_storage(
                shape,
                identity,
                combine,
                shape_metrics,
                frontier,
                outcomes.values,
                failed,
                storage,
                |charge| {
                    if !accounted {
                        context.poll_frontier()
                    } else {
                        match charge {
                            FrontierCharge::Node => context.checkpoint_frontier(1),
                            FrontierCharge::CompletedNode
                            | FrontierCharge::PrefixJoin
                            | FrontierCharge::TaskUnit => {
                                // Charge the canonical prefix, including joins before a known
                                // stopped task. The stop keeps its canonical position;
                                // refused charges still stop before the join runs.
                                context.account_completed_work(1)?;
                                if matches!(charge, FrontierCharge::TaskUnit) {
                                    let accepted =
                                        accepted_task_work.as_mut().expect("accounted frontier");
                                    *accepted = accepted
                                        .checked_add(1)
                                        .ok_or_else(KernelStop::work_counter_overflow)?;
                                }
                                Ok(())
                            }
                        }
                    }
                },
            );
            if let Some(accepted) = accepted_task_work {
                if !context.reconcile_discarded_work(outcomes.report.charged_work(), accepted) {
                    return Err(metrics_mismatch(shape_metrics));
                }
            }
            let (tree, metrics) = settled.map_err(|mut failure| {
                // The serial checked oracle reports an interrupted traversal's
                // accepted prefix as sequential span. Completed speculative
                // subtrees may have DAG spans, but they cannot change this
                // failure-prefix convention when frontier width changes.
                if let Some(metrics) = failure.metrics.accounted_mut() {
                    metrics.charged_span = metrics.charged_work;
                }
                ScheduledReductionError::Reduction(failure)
            })?;
            if accepted_task_work.is_some_and(|accepted| outcomes.report.charged_work() != accepted)
            {
                return Err(metrics_mismatch(metrics));
            }
            if let Some(before_evaluation) = before_evaluation {
                let accounting = metrics.accounted_metrics().expect("accounted frontier");
                let shape_accounting = shape_metrics.accounted_metrics().expect("accounted shape");
                let evaluation_work = accounting
                    .charged_work
                    .checked_sub(shape_accounting.charged_work)
                    .ok_or_else(|| metrics_mismatch(metrics))?;
                let evaluation_span = accounting
                    .charged_span
                    .checked_sub(shape_accounting.charged_span)
                    .ok_or_else(|| metrics_mismatch(metrics))?;
                context
                    .apply_structural_span(before_evaluation, evaluation_work, evaluation_span)
                    .map_err(|failure| {
                        ScheduledReductionError::Reduction(
                            ReductionRunFailure::structural_span_failure(failure, metrics),
                        )
                    })?;
            }
            Ok((tree, metrics))
        })()
    }
}

fn task_failure(
    cause: KernelFailure<ReductionRunFailure<KernelStop>>,
    completed: ReductionMetrics,
) -> ReductionRunFailure<KernelStop> {
    let reason = match cause {
        KernelFailure::Domain(failure) => return failure,
        KernelFailure::Stop(stop) => ReductionRunStop::Hook(stop),
        KernelFailure::Panic => ReductionRunStop::Panic,
        KernelFailure::ResultCapacityExceeded => ReductionRunStop::ResultCapacityExceeded,
    };
    ReductionRunFailure {
        reason,
        metrics: completed,
    }
}

/// Inline task state survives replacement of the worker result by a request stop.
/// Its full size, including the metric cell, is reserved by batch admission.
pub(super) struct ScheduledTask {
    spec: SubtreeSpec,
    completed: Option<OnceLock<ReductionMetrics>>,
}

impl ChargedBytes for ScheduledTask {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { spec, completed } = self;
        // Neither the subtree coordinates nor these scalar metrics own heap bytes.
        let _ = (spec, completed);
        0
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
        reason: ReductionRunStop::work_counter_overflow(),
        metrics,
    })
}

fn metrics_mismatch(metrics: ReductionMetrics) -> ScheduledReductionError {
    ScheduledReductionError::Reduction(ReductionRunFailure::structural_span_failure(
        StructuralSpanFailure::MetricsMismatch,
        metrics,
    ))
}

#[cfg(test)]
mod tests;
