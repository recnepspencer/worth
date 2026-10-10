use std::mem::size_of;

use worth_foundational::{
    ExecutionPhysicalReport, ExecutionPosture, ExecutionReport, PartitionIdentity,
};

use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial},
    backend::{
        run_scope_holding, BackendKind, KernelContext, KernelFailure, KernelStop, ScopeHold,
        ScopeStop,
    },
    oracle::CanonicalBits,
    reduction::{
        ReductionMetrics, ReductionPlan, ReductionRunFailure, ReductionRunStop, ReductionTree,
    },
    report::ChargedBytes,
};

use super::{ExecutionMap, MapOutcome, MapStop};

mod certify;
mod entry;
mod serial;
mod stage;
mod stop_owner;
pub use certify::ReduceCertificationFailure;

pub type ExecutionReduce<T, F> = ReductionTree<T, F>;

#[derive(Debug)]
pub enum ReduceInputDenial<E> {
    ScopeAdmission {
        denial: LeaseDenial,
        report: ExecutionReport,
    },
    MapStopped {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
    ReductionStopped {
        failure: ReductionRunFailure<KernelStop>,
        report: ExecutionReport,
    },
}

pub(super) struct ReduceComplete<R, F> {
    pub(super) tree: ReductionTree<R, F>,
    pub(super) metrics: ReductionMetrics,
}

impl<R: ChargedBytes, F> ChargedBytes for ReduceComplete<R, F> {
    fn additional_charged_bytes(&self) -> u64 {
        self.tree.additional_charged_bytes()
    }
}

pub(super) enum ReduceScopeError {
    Admission(LeaseDenial),
    Reduction(ReductionRunFailure<KernelStop>),
}

impl ChargedBytes for ReduceScopeError {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    fn run_reduce_leased<R, E, Kernel, Combine>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
        backend: BackendKind,
        charge_parent: bool,
        mut retained_handoff: Option<&mut dyn FnMut(u64) -> Result<(), LeaseDenial>>,
        inputs: Option<ExecutionMemoryReservation>,
        tree: Option<&mut ExecutionMemoryReservation>,
    ) -> Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        let count = self.partition_count();
        let tree_bytes =
            ReductionTree::<R, Combine>::checked_build_memory_bound(count, max_value_bytes)
                .ok_or_else(memory_admission_denial)?;
        let plan_bytes = count
            .checked_mul(size_of::<PartitionIdentity>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(memory_admission_denial)?;
        let retained_bytes = self
            .retained_result_bytes::<R>()
            .and_then(|bytes| bytes.checked_add(plan_bytes))
            .and_then(|bytes| bytes.checked_add(reducer_storage_bytes))
            .ok_or_else(memory_admission_denial)?;
        let mut map_stopped = None;
        let mut map_fallback = None;
        let mut reduction_checkpoint_failure = None;
        let kept = |complete: &ReduceComplete<R, Combine>| {
            kept_tree_bytes(&complete.tree, reducer_storage_bytes)
        };
        let hold = tree.map(|reservation| ScopeHold {
            reservation,
            kept: &kept,
        });
        let mut outcome = run_scope_holding(
            Some(lease),
            retained_bytes,
            tree_bytes,
            charge_parent,
            hold,
            |context| {
                let mapped = self.run_with_backend_taking(Some(lease), backend, inputs, kernel);
                map_fallback = mapped.report().fallback();
                let values = match mapped {
                    MapOutcome::Complete { values, .. } => values,
                    MapOutcome::Stopped {
                        boundary, reason, ..
                    } => {
                        map_stopped = Some((boundary, reason));
                        return Err(KernelFailure::Stop(KernelStop::NestedStopped));
                    }
                };
                context.checkpoint(0)?;
                let plan = ReductionPlan::try_from_sorted_unique(self.identities().to_vec())
                    .map_err(|denial| {
                        KernelFailure::Domain(ReduceScopeError::Reduction(ReductionRunFailure {
                            reason: ReductionRunStop::Denial(denial),
                            metrics: ReductionMetrics::default(),
                        }))
                    })?;
                let complete = match stage::execute(
                    lease,
                    backend,
                    plan,
                    values,
                    identity,
                    combine,
                    max_value_bytes,
                    context,
                ) {
                    Ok(complete) => complete,
                    Err(ReduceScopeError::Reduction(failure)) => {
                        stop_owner::record_reduction(context, &failure.reason);
                        if let ReductionRunStop::Hook(stop) = &failure.reason {
                            let stop = *stop;
                            reduction_checkpoint_failure = Some(failure);
                            return Err(KernelFailure::Stop(stop));
                        }
                        return Err(KernelFailure::Domain(ReduceScopeError::Reduction(failure)));
                    }
                    Err(error) => {
                        error.record_owner(context);
                        return Err(KernelFailure::Domain(error));
                    }
                };
                if let Some(handoff) = retained_handoff.as_mut() {
                    let bytes = kept_tree_bytes(&complete.tree, reducer_storage_bytes).ok_or_else(
                        || {
                            context
                                .record_nested_stop_owner(crate::backend::NestedStopOwner::Request);
                            KernelFailure::Domain(ReduceScopeError::Admission(
                                LeaseDenial::charged_bytes_overflow(),
                            ))
                        },
                    )?;
                    handoff(bytes).map_err(|denial| {
                        context.record_nested_stop_owner(crate::backend::NestedStopOwner::Request);
                        KernelFailure::Domain(ReduceScopeError::Admission(denial))
                    })?;
                }
                Ok(complete)
            },
        );
        // The scope reports how the reduction ran; why its map ran serially is
        // the map's to name.
        if let Some(cause) = map_fallback {
            outcome.report = outcome.report.with_fallback(cause);
        }
        match outcome.result {
            Ok(complete) => Ok((complete.tree, outcome.report, complete.metrics)),
            Err(ScopeStop::Admission(denial)) => Err(ReduceInputDenial::ScopeAdmission {
                denial,
                report: outcome.report,
            }),
            Err(ScopeStop::Failure(KernelFailure::Domain(ReduceScopeError::Reduction(
                failure,
            )))) => Err(ReduceInputDenial::ReductionStopped {
                failure,
                report: outcome.report,
            }),
            Err(ScopeStop::Failure(KernelFailure::Domain(ReduceScopeError::Admission(denial)))) => {
                Err(ReduceInputDenial::ScopeAdmission {
                    denial,
                    report: outcome.report,
                })
            }
            Err(ScopeStop::Failure(_)) if map_stopped.is_some() => {
                let (boundary, reason) = map_stopped.expect("checked map stop");
                Err(ReduceInputDenial::MapStopped {
                    boundary,
                    reason,
                    report: outcome.report,
                })
            }
            Err(ScopeStop::Failure(cause)) => {
                let failure = reduction_checkpoint_failure.unwrap_or_else(|| ReductionRunFailure {
                    reason: scope_reduction_reason(cause),
                    metrics: ReductionMetrics::default(),
                });
                Err(ReduceInputDenial::ReductionStopped {
                    failure,
                    report: outcome.report,
                })
            }
        }
    }
}

/// What a completed tree keeps once its run ends: its nodes and identity,
/// the reducer's declared storage, and the tree itself.
fn kept_tree_bytes<R: ChargedBytes, Combine>(
    tree: &ReductionTree<R, Combine>,
    reducer_storage_bytes: u64,
) -> Option<u64> {
    tree.additional_charged_bytes()
        .checked_add(reducer_storage_bytes)?
        .checked_add(u64::try_from(size_of::<ReductionTree<R, Combine>>()).ok()?)
}

fn scope_reduction_reason<E>(cause: KernelFailure<E>) -> ReductionRunStop<KernelStop> {
    match cause {
        KernelFailure::Stop(stop) => ReductionRunStop::Hook(stop),
        KernelFailure::Panic => ReductionRunStop::Panic,
        KernelFailure::ResultCapacityExceeded => ReductionRunStop::ResultCapacityExceeded,
        KernelFailure::Domain(_) => unreachable!("scope domain failures handled above"),
    }
}

fn memory_admission_denial<E>() -> ReduceInputDenial<E> {
    let report = ExecutionReport::new(
        ExecutionPosture::Serial,
        0,
        0,
        ExecutionPhysicalReport::default(),
    );
    crate::backend::record_nested(report, Some(crate::backend::NestedStopOwner::Request));
    ReduceInputDenial::ScopeAdmission {
        denial: LeaseDenial::charged_bytes_overflow(),
        report,
    }
}
