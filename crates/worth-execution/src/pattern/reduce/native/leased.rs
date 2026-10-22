use super::*;
use crate::{
    backend::{run_scope_native_within, NestedStopOwner},
    reduction::{RunTree, ScheduledReductionError},
    ExecutionRetainedRun, NativeReductionTree, RetainedRunMetrics, RetainedRunOutcome,
};

pub(super) struct NativeCapture<R> {
    pub(super) run: Option<ExecutionRetainedRun<R>>,
    pub(super) denial: Option<RetainedMapDenial>,
    pub(super) report: Option<ExecutionReport>,
}
impl<R> Default for NativeCapture<R> {
    fn default() -> Self {
        Self {
            run: None,
            denial: None,
            report: None,
        }
    }
}

impl<T: Sync, K> ExecutionMap<T, K, NativePayload> {
    pub(super) fn run_reduce_native_leased<R, E, Kernel, Combine>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        charge_parent: bool,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        capture: &mut NativeCapture<R>,
    ) -> Result<
        (
            NativeReductionTree<R, Combine>,
            ExecutionReport,
            ReductionMetrics,
        ),
        ReduceInputDenial<E>,
    >
    where
        R: Clone + Send + Sync + CanonicalBits,
        E: Send,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        let mut map_stopped = None;
        let mut reduction_checkpoint_failure = None;
        let outcome = run_scope_native_within(lease, charge_parent, None, None, |context| {
            let mut mapped = self
                .run_retained_native_with_backend(lease, backend, &kernel)
                .map_err(|cause| {
                    let cause = RetainedMapDenial::storage(cause);
                    let owner = match &cause {
                        RetainedMapDenial::Request(_) | RetainedMapDenial::Admission(_) => {
                            NestedStopOwner::Request
                        }
                        RetainedMapDenial::Allocation(cause) => match cause.classify() {
                            crate::ExecutionAllocationDenialClass::Request(_) => {
                                NestedStopOwner::Request
                            }
                            crate::ExecutionAllocationDenialClass::Physical(kind) => {
                                NestedStopOwner::Computation(
                                    crate::backend::NestedComputationStop::Allocation(kind),
                                )
                            }
                        },
                        RetainedMapDenial::RequiresExecution(_) => NestedStopOwner::Computation(
                            crate::backend::NestedComputationStop::DomainFailure,
                        ),
                    };
                    capture.denial = Some(cause);
                    context.record_nested_stop_owner(owner);
                    KernelFailure::Stop(KernelStop::NestedStopped)
                })?;
            capture.report = Some(mapped.report);
            if let Some(reason) = mapped.stop.take() {
                map_stopped = Some((mapped.prefix_boundary, reason));
                return Err(KernelFailure::Stop(KernelStop::NestedStopped));
            }
            context.poll()?;
            let (mut values, _values_memory) = buffer::<R>(mapped.values.len(), lease, context)?;
            let (mut ids, _ids_memory) =
                buffer::<PartitionIdentity>(self.partition_count(), lease, context)?;
            values.extend(mapped.values.iter().map(|value| value.value().clone()));
            ids.extend_from_slice(self.identities());
            if self.has_keyless_retention() {
                let completed = super::super::super::map::finish_native_outcome(
                    RunTree::empty(),
                    mapped,
                    lease,
                    RetainedRunMetrics {
                        supplied_inputs: self.partition_count() as u64,
                        ..Default::default()
                    },
                )
                .map_err(|cause| {
                    capture.denial = Some(RetainedMapDenial::Admission(cause));
                    context.record_nested_stop_owner(NestedStopOwner::Request);
                    KernelFailure::Stop(KernelStop::NestedStopped)
                })?;
                let RetainedRunOutcome::Complete { run, .. } = completed else {
                    unreachable!("only completed native map enters custody")
                };
                capture.run = Some(run);
            }
            let plan = ReductionPlan::try_from_sorted_unique(ids).map_err(|denial| {
                KernelFailure::Domain(ReduceScopeError::Reduction(ReductionRunFailure {
                    reason: ReductionRunStop::Denial(denial),
                    metrics: ReductionMetrics::for_mode(context.has_accounting()),
                }))
            })?;
            let reduced = match lease {
                Some(lease) => ReductionTree::build_scheduled_native(
                    lease, backend, plan, values, identity, combine, context,
                )
                .map_err(|cause| match cause {
                    ScheduledReductionError::Admission(cause) => ReduceScopeError::Admission(cause),
                    ScheduledReductionError::Reduction(cause) => ReduceScopeError::Reduction(cause),
                }),
                None => serial(plan, values, identity, combine, context)
                    .map_err(ReduceScopeError::Reduction),
            };
            match reduced {
                Ok((tree, metrics)) => Ok((tree, metrics)),
                Err(ReduceScopeError::Reduction(failure)) => {
                    stop_owner::record_reduction(context, &failure.reason);
                    if let ReductionRunStop::Hook(stop) = failure.reason {
                        reduction_checkpoint_failure = Some(failure);
                        Err(KernelFailure::Stop(stop))
                    } else {
                        Err(KernelFailure::Domain(ReduceScopeError::Reduction(failure)))
                    }
                }
                Err(cause) => {
                    cause.record_owner(context);
                    Err(KernelFailure::Domain(cause))
                }
            }
        });
        let report = capture
            .report
            .and_then(|report| report.fallback())
            .map_or(outcome.report, |cause| outcome.report.with_fallback(cause));
        match outcome.result {
            Ok((tree, metrics)) => Ok((tree, report, metrics)),
            Err(ScopeStop::Admission(denial)) => {
                Err(ReduceInputDenial::ScopeAdmission { denial, report })
            }
            Err(ScopeStop::Failure(KernelFailure::Domain(ReduceScopeError::Admission(denial)))) => {
                Err(ReduceInputDenial::ScopeAdmission { denial, report })
            }
            Err(ScopeStop::Failure(KernelFailure::Domain(ReduceScopeError::Reduction(
                failure,
            )))) => Err(ReduceInputDenial::ReductionStopped { failure, report }),
            Err(_) if map_stopped.is_some() => {
                let (boundary, reason) = map_stopped.expect("checked map stop");
                Err(ReduceInputDenial::MapStopped {
                    boundary,
                    reason,
                    report,
                })
            }
            Err(ScopeStop::Failure(cause)) => Err(ReduceInputDenial::ReductionStopped {
                failure: reduction_checkpoint_failure.unwrap_or_else(|| ReductionRunFailure {
                    reason: scope_reduction_reason(cause),
                    metrics: ReductionMetrics::for_mode(report.has_accounting()),
                }),
                report,
            }),
        }
    }
}

fn serial<R: Clone + CanonicalBits, F: Fn(&R, &R) -> R>(
    plan: ReductionPlan,
    values: Vec<R>,
    identity: R,
    combine: F,
    context: &mut KernelContext<'_, '_>,
) -> Result<(NativeReductionTree<R, F>, ReductionMetrics), ReductionRunFailure<KernelStop>> {
    let before = context.has_accounting().then(|| context.cost());
    let result = ReductionTree::from_plan_native(plan, values, identity, combine, None, || {
        if before.is_some() {
            context.checkpoint(1)
        } else {
            context.poll()
        }
    });
    let metrics = match &result {
        Ok((_, metrics)) => *metrics,
        Err(failure) => failure.metrics,
    };
    if let Some(before) = before {
        let accounting = metrics
            .accounted_metrics()
            .expect("accounted reducer context");
        context
            .apply_structural_span(before, accounting.charged_work, accounting.charged_span)
            .map_err(|cause| ReductionRunFailure::structural_span_failure(cause, metrics))?;
    }
    result
}

fn allocation(
    context: &mut KernelContext<'_, '_>,
    kind: crate::ExecutionPhysicalAllocationDenialKind,
) -> KernelFailure<ReduceScopeError> {
    let failure = ReductionRunFailure {
        reason: ReductionRunStop::Denial(crate::reduction::ReductionDenial::Allocation(kind)),
        metrics: ReductionMetrics::for_mode(context.has_accounting()),
    };
    stop_owner::record_reduction(context, &failure.reason);
    KernelFailure::Domain(ReduceScopeError::Reduction(failure))
}

fn buffer<T>(
    length: usize,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut KernelContext<'_, '_>,
) -> Result<(Vec<T>, Option<crate::ExecutionMemoryReservation>), KernelFailure<ReduceScopeError>> {
    use crate::ExecutionPhysicalAllocationDenialKind as Kind;
    let layout =
        std::alloc::Layout::array::<T>(length).map_err(|_| allocation(context, Kind::Layout))?;
    crate::backend::RunLimits::check_native_scope(lease).map_err(|cause| {
        context.record_nested_stop_owner(NestedStopOwner::Request);
        KernelFailure::Domain(ReduceScopeError::Admission(cause))
    })?;
    let memory = if !context.has_accounting() || layout.size() == 0 {
        None
    } else {
        let bytes = u64::try_from(layout.size()).map_err(|_| allocation(context, Kind::Layout))?;
        Some(
            crate::ExecutionMemoryReservation::reserve_in_scope(lease, bytes).map_err(|cause| {
                context.record_nested_stop_owner(NestedStopOwner::Request);
                KernelFailure::Domain(ReduceScopeError::Admission(cause))
            })?,
        )
    };
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| allocation(context, Kind::Allocator))?;
    Ok((values, memory))
}
