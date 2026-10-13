//! The bounded serial backing retains the reducer's cause and metrics.
use super::*;

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    pub(super) fn run_reduce_serial<R, E, Kernel, Combine>(
        &self,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
        inputs: Option<ExecutionMemoryReservation>,
        tree: Option<&mut ExecutionMemoryReservation>,
    ) -> Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>
    where
        R: Send + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R,
    {
        let tree_bytes = ReductionTree::<R, Combine>::checked_serial_build_memory_bound(
            self.partition_count(),
            max_value_bytes,
        )
        .ok_or_else(memory_admission_denial)?;
        let mut map_stopped = None;
        let mut map_fallback = None;
        let mut reduction_failure = None;
        let kept = |complete: &ReduceComplete<R, Combine>| {
            kept_tree_bytes(&complete.tree, reducer_storage_bytes)
        };
        let hold = tree.map(|reservation| ScopeHold {
            reservation,
            kept: &kept,
        });
        let outcome = run_scope_holding::<ReduceComplete<R, Combine>, ReduceScopeError, _>(
            None,
            0,
            tree_bytes,
            true,
            hold,
            |context| {
                let values =
                    match self.run_with_backend_taking(None, BackendKind::Serial, inputs, kernel) {
                        MapOutcome::Complete { values, report } => {
                            map_fallback = report.fallback();
                            values
                        }
                        MapOutcome::Stopped {
                            boundary, reason, ..
                        } => {
                            map_stopped = Some((boundary, reason));
                            return Err(KernelFailure::Stop(KernelStop::NestedStopped));
                        }
                    };
                context.checkpoint(0)?;
                let plan = match ReductionPlan::try_from_sorted_unique(self.identities().to_vec()) {
                    Ok(plan) => plan,
                    Err(denial) => {
                        return Err(KernelFailure::Domain(ReduceScopeError::Reduction(
                            ReductionRunFailure {
                                reason: ReductionRunStop::Denial(denial),
                                metrics: ReductionMetrics::default(),
                            },
                        )));
                    }
                };
                let before = context.cost();
                let reduced = ReductionTree::from_plan_checked(
                    plan,
                    values,
                    identity,
                    combine,
                    max_value_bytes,
                    || context.checkpoint(1),
                );
                let metrics = match &reduced {
                    Ok((_, metrics)) => *metrics,
                    Err(failure) => failure.metrics,
                };
                if let Err(failure) = context.apply_structural_span(
                    before,
                    metrics.charged_work,
                    metrics.charged_span,
                ) {
                    return Err(KernelFailure::Domain(ReduceScopeError::Reduction(
                        ReductionRunFailure::structural_span_failure(failure, metrics),
                    )));
                }
                match reduced {
                    Ok((tree, metrics)) => Ok(ReduceComplete { tree, metrics }),
                    Err(failure) => {
                        stop_owner::record_reduction(context, &failure.reason);
                        if let ReductionRunStop::Hook(stop) = failure.reason {
                            reduction_failure = Some(failure);
                            Err(KernelFailure::Stop(stop))
                        } else {
                            Err(KernelFailure::Domain(ReduceScopeError::Reduction(failure)))
                        }
                    }
                }
            },
        );
        let report =
            map_fallback.map_or(outcome.report, |cause| outcome.report.with_fallback(cause));
        match outcome.result {
            Ok(complete) => Ok((complete.tree, report, complete.metrics)),
            Err(ScopeStop::Admission(denial)) => {
                Err(ReduceInputDenial::ScopeAdmission { denial, report })
            }
            Err(_) if map_stopped.is_some() => {
                let (boundary, reason) = map_stopped.expect("checked map stop");
                Err(ReduceInputDenial::MapStopped {
                    boundary,
                    reason,
                    report,
                })
            }
            Err(ScopeStop::Failure(KernelFailure::Domain(ReduceScopeError::Reduction(
                failure,
            )))) => Err(ReduceInputDenial::ReductionStopped { failure, report }),
            Err(cause) => Err(ReduceInputDenial::ReductionStopped {
                failure: reduction_failure.unwrap_or_else(|| ReductionRunFailure {
                    reason: match cause {
                        ScopeStop::Failure(cause) => scope_reduction_reason(cause),
                        ScopeStop::Admission(_) => unreachable!("admission handled above"),
                    },
                    metrics: ReductionMetrics::default(),
                }),
                report,
            }),
        }
    }
}
