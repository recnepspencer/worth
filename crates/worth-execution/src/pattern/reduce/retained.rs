//! Canonical reduction and its genuine, independently funded native leaves.
use super::*;
use crate::{ExecutionRequest, ExecutionRetainedResult, ExecutionWorkCeiling, RetainedMapDenial};

pub struct RetainedReduction<R, F> {
    pub tree: ReductionTree<R, F>,
    pub partition_results: Vec<ExecutionRetainedResult<R>>,
    pub report: ExecutionReport,
    pub metrics: ReductionMetrics,
}
/// The map's already settled report remains available on a later tree refusal.
/// Observers do not need to ask a live kernel context about its remaining work.
pub struct RetainedReduceOutcome<R, F, E> {
    pub result: Result<RetainedReduction<R, F>, RetainedReduceDenial<E>>,
    pub mapped_report: Option<ExecutionReport>,
    pub reduction_metrics: Option<ReductionMetrics>,
}
#[derive(Debug)]
pub enum RetainedReduceDenial<E> {
    Retention(RetainedMapDenial),
    Reduction(ReduceInputDenial<E>),
}

pub(super) struct RetentionCapture<R> {
    results: Vec<ExecutionRetainedResult<R>>,
    denial: Option<RetainedMapDenial>,
    report: Option<ExecutionReport>,
}
impl<R: Clone + ChargedBytes + Send + Sync> RetentionCapture<R> {
    pub(super) fn map<T: ChargedBytes + Sync, K, E: ChargedBytes + Send, F>(
        &mut self,
        map: &ExecutionMap<T, K>,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        inputs: Option<ExecutionMemoryReservation>,
        kernel: F,
        context: &mut KernelContext<'_, '_>,
    ) -> Result<MapOutcome<R, E>, KernelFailure<ReduceScopeError>>
    where
        F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
    {
        let outcome = map
            .run_retained_with_backend_taking(lease, backend, inputs, &kernel)
            .map_err(|cause| {
                let cause = RetainedMapDenial::storage(cause);
                use crate::backend::{NestedComputationStop as Cause, NestedStopOwner as Owner};
                let owner = match &cause {
                    RetainedMapDenial::Admission(_) | RetainedMapDenial::Request(_) => {
                        Owner::Request
                    }
                    RetainedMapDenial::Allocation(cause) => match cause.classify() {
                        crate::ExecutionAllocationDenialClass::Request(_) => Owner::Request,
                        crate::ExecutionAllocationDenialClass::Physical(_) => {
                            Owner::Computation(Cause::DomainFailure)
                        }
                    },
                    RetainedMapDenial::RequiresExecution(_) => {
                        Owner::Computation(Cause::DomainFailure)
                    }
                };
                context.record_nested_stop_owner(owner);
                self.denial = Some(cause);
                KernelFailure::Stop(KernelStop::NestedStopped)
            })?;
        self.report = Some(outcome.report);
        Ok(match MapOutcome::from(outcome) {
            MapOutcome::Complete { values, report } => {
                // The reduction scope has already admitted its full input and
                // tree bounds. New native values remain independently funded
                // while these tree inputs are copied, including old/new peak.
                let copied = values.iter().map(|value| value.value().clone()).collect();
                self.results = values;
                MapOutcome::Complete {
                    values: copied,
                    report,
                }
            }
            MapOutcome::Stopped {
                boundary,
                reason,
                report,
                ..
            } => MapOutcome::Stopped {
                completed_prefix: Vec::new(),
                boundary,
                reason,
                report,
            },
        })
    }
}

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    /// Full checked mapping plus canonical reduction, retaining genuine native
    /// leaves only when the entire reduction completes. Input/tree handoff is
    /// the same atomic reservation takeover as `run_reduce_holding`.
    pub fn run_reduce_retained_holding<R, E, Kernel, Combine>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        inputs: Option<ExecutionMemoryReservation>,
        mut tree: Option<&mut ExecutionMemoryReservation>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> RetainedReduceOutcome<R, Combine, E>
    where
        R: Clone + Send + Sync + ChargedBytes + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        let mut capture = RetentionCapture {
            results: Vec::new(),
            denial: None,
            report: None,
        };
        let run = request.run(ceiling, |lease| match lease {
            Some(lease) => self.run_reduce_leased(
                lease,
                kernel,
                identity,
                combine,
                max_value_bytes,
                reducer_storage_bytes,
                super::super::map::backend_for(Some(lease)),
                true,
                None,
                inputs,
                tree.as_deref_mut(),
                Some(&mut capture),
            ),
            None => self.run_reduce_serial(
                kernel,
                identity,
                combine,
                max_value_bytes,
                reducer_storage_bytes,
                inputs,
                tree.as_deref_mut(),
                Some(&mut capture),
            ),
        });
        let tree_handoff_succeeded = run.as_ref().is_ok_and(|(result, _)| result.is_ok());
        let reduction_metrics = run.as_ref().ok().and_then(|(result, _)| match result {
            Ok((_, _, metrics)) => Some(*metrics),
            Err(ReduceInputDenial::ReductionStopped { failure, .. }) => Some(failure.metrics),
            Err(_) => None,
        });
        let result = (|| {
            let result = run
                .map_err(|cause| {
                    RetainedReduceDenial::Retention(RetainedMapDenial::Request(cause))
                })?
                .0;
            request.check_live().map_err(|stop| {
                RetainedReduceDenial::Retention(RetainedMapDenial::interruption(stop))
            })?;
            if let Some(denial) = capture.denial {
                return Err(RetainedReduceDenial::Retention(denial));
            }
            let (tree, report, metrics) = result.map_err(RetainedReduceDenial::Reduction)?;
            request.check_live().map_err(|stop| {
                RetainedReduceDenial::Retention(RetainedMapDenial::interruption(stop))
            })?;
            Ok(RetainedReduction {
                tree,
                partition_results: capture.results,
                report,
                metrics,
            })
        })();
        // Clear only the target this completed native run actually took over.
        // The rejected tree dropped inside the closure before its grant shrinks.
        if result.is_err()
            && tree_handoff_succeeded
            && let Some(tree) = tree
        {
            tree.resize(0).expect("releasing a rejected completed tree");
        }
        RetainedReduceOutcome {
            result,
            mapped_report: capture.report,
            reduction_metrics,
        }
    }
}
