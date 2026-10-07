use std::mem::size_of;

use worth_foundational::{
    ExecutionPhysicalReport, ExecutionPosture, ExecutionReport, PartitionIdentity,
};

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::{
        run_scope, run_scope_with_charge, BackendKind, KernelContext, KernelFailure, KernelStop,
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
mod stage;
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
    /// Map values stay bound to checked identities while one parent scope
    /// retains their memory through canonical tree construction. `max_value_bytes`
    /// bounds each reducer value's inline and owned bytes and encoded length;
    /// `reducer_storage_bytes` declares heap storage captured by `combine`.
    pub fn run_reduce<R, E, Kernel, Combine>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        if let Some(lease) = lease {
            self.run_reduce_leased(
                lease,
                kernel,
                identity,
                combine,
                max_value_bytes,
                reducer_storage_bytes,
                if lease.resolved_posture() == ExecutionPosture::Automatic {
                    BackendKind::Native
                } else {
                    BackendKind::Serial
                },
                true,
                None,
            )
        } else {
            self.run_reduce_serial(kernel, identity, combine, max_value_bytes)
        }
    }

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
        let mut reduction_checkpoint_failure = None;
        let outcome = run_scope_with_charge(
            Some(lease),
            retained_bytes,
            tree_bytes,
            charge_parent,
            |context| {
                let values = match self.run_with_backend(Some(lease), backend, kernel) {
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
                        if let ReductionRunStop::Hook(stop) = &failure.reason {
                            let stop = *stop;
                            reduction_checkpoint_failure = Some(failure);
                            return Err(KernelFailure::Stop(stop));
                        }
                        return Err(KernelFailure::Domain(ReduceScopeError::Reduction(failure)));
                    }
                    Err(error) => return Err(KernelFailure::Domain(error)),
                };
                if let Some(handoff) = retained_handoff.as_mut() {
                    let bytes = complete
                        .tree
                        .additional_charged_bytes()
                        .checked_add(reducer_storage_bytes)
                        .and_then(|bytes| {
                            bytes.checked_add(
                                u64::try_from(size_of::<ReductionTree<R, Combine>>()).ok()?,
                            )
                        })
                        .ok_or({
                            KernelFailure::Domain(ReduceScopeError::Admission(
                                LeaseDenial::ResourceExhausted,
                            ))
                        })?;
                    handoff(bytes).map_err(|denial| {
                        KernelFailure::Domain(ReduceScopeError::Admission(denial))
                    })?;
                }
                Ok(complete)
            },
        );
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

    fn run_reduce_serial<R, E, Kernel, Combine>(
        &self,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
    ) -> Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>
    where
        R: Send + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R,
    {
        let tree_bytes = ReductionTree::<R, Combine>::checked_build_memory_bound(
            self.partition_count(),
            max_value_bytes,
        )
        .ok_or_else(memory_admission_denial)?;
        let mut map_stopped = None;
        let mut map_fallback = None;
        let mut reduction_failure = None;
        let outcome = run_scope::<ReduceComplete<R, Combine>, ReduceScopeError, _>(
            None,
            0,
            tree_bytes,
            |context| {
                let values = match self.run(None, kernel) {
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
                        reduction_failure = Some(ReductionRunFailure {
                            reason: ReductionRunStop::Denial(denial),
                            metrics: ReductionMetrics::default(),
                        });
                        return Err(KernelFailure::Stop(KernelStop::NestedStopped));
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
                if !context.apply_structural_span(
                    before,
                    metrics.charged_work,
                    metrics.charged_span,
                ) {
                    reduction_failure = Some(ReductionRunFailure {
                        reason: ReductionRunStop::WorkCounterOverflow,
                        metrics,
                    });
                    return Err(KernelFailure::Stop(KernelStop::WorkCounterOverflow));
                }
                match reduced {
                    Ok((tree, metrics)) => Ok(ReduceComplete { tree, metrics }),
                    Err(failure) => {
                        let stop = match failure.reason {
                            ReductionRunStop::Hook(stop) => stop,
                            _ => KernelStop::NestedStopped,
                        };
                        reduction_failure = Some(failure);
                        Err(KernelFailure::Stop(stop))
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

fn scope_reduction_reason<E>(cause: KernelFailure<E>) -> ReductionRunStop<KernelStop> {
    match cause {
        KernelFailure::Stop(stop) => ReductionRunStop::Hook(stop),
        KernelFailure::Panic => ReductionRunStop::Panic,
        KernelFailure::ResultCapacityExceeded => ReductionRunStop::ResultCapacityExceeded,
        KernelFailure::Domain(_) => unreachable!("scope domain failures handled above"),
    }
}

fn memory_admission_denial<E>() -> ReduceInputDenial<E> {
    ReduceInputDenial::ScopeAdmission {
        denial: LeaseDenial::ResourceExhausted,
        report: ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        ),
    }
}
