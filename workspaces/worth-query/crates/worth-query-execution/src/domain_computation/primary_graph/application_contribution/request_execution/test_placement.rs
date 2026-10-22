//! A test's choice of where the requests its thread opens run their managed
//! computations, in place of the World's placement: serially, on a lease of
//! a chosen worker count, or on such a lease with each full reduction also
//! certified against the serial oracle and a seeded perturbed backend.
//!
//! Every lease is drawn from the one authority a process may construct. It
//! admits twice the machine's width, so a lease may ask for more workers than
//! the machine has. Memory proofs declare a finite request budget; ordinary
//! test requests keep memory accounting without imposing a fixture cap.

use super::test_authority::query_test_execution_authority_for_test;
mod native_certification;
pub use super::test_authority::test_execution_workers_for_test;
pub(super) use native_certification::certify_native;
use std::cell::Cell;
use std::num::NonZeroUsize;

use worth_execution::{
    compare_canonical_values, CanonicalBits, ExecutionMap, ExecutionResourceLease,
    MapKernelContext, MapKernelFailure, ReduceCertificationFailure,
};
#[cfg(test)]
use worth_execution::{ChargedBytes, ReductionTree};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionReport, ExecutionRequestPolicy,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

/// Where the requests a test's thread opens run their managed computations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionPlacementForTest {
    /// Where the World places them.
    World,
    /// Serially, within a policy whose memory is a lease's.
    Serial,
    /// On a lease of this many workers.
    Leased(NonZeroUsize),
    /// On a lease of `workers`, each completed full reduction then certified
    /// against the serial oracle and the perturbed backend of `seed`.
    Certified { workers: NonZeroUsize, seed: u64 },
}

use crate::domain_computation::execution_observation::owned_control_scope;

thread_local! {
    static PLACEMENT_OWNER: Cell<u64> = const { Cell::new(0) };
    static BUDGET_OWNER: Cell<u64> = const { Cell::new(0) };
    static REQUEST_BUDGET: Cell<Option<ExecutionBudget>> = const { Cell::new(None) };
    static PLACEMENT: Cell<WorthQueryExecutionPlacementForTest> =
        const { Cell::new(WorthQueryExecutionPlacementForTest::World) };
}

/// Places the managed computations of the requests this thread opens from
/// now on, and returns the placement it replaces.
pub fn place_managed_computations_on_this_thread_for_test(
    placement: WorthQueryExecutionPlacementForTest,
) -> WorthQueryExecutionPlacementForTest {
    // Law 2, scope: this pre-request placement is owned by the arming guard.
    let owner = owned_control_scope::current_id();
    if PLACEMENT_OWNER.get() != owner {
        let previous = (PLACEMENT.get(), PLACEMENT_OWNER.get());
        owned_control_scope::register(move || {
            if PLACEMENT_OWNER.get() == owner {
                let (placement, owner) = if owned_control_scope::is_live(previous.1) {
                    previous
                } else {
                    (WorthQueryExecutionPlacementForTest::World, 0)
                };
                PLACEMENT.set(placement);
                PLACEMENT_OWNER.set(owner);
            }
        });
    }
    PLACEMENT_OWNER.set(owner);
    PLACEMENT.replace(placement)
}

/// A canonical, automatic policy of `workers` and `memory`, with no work
/// limit beyond each computation's own.
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn test_policy(
    workers: NonZeroUsize,
    memory: Option<u64>,
) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        REQUEST_BUDGET
            .get()
            .map_or(ExecutionBudget::new(workers, memory, None), |budget| {
                ExecutionBudget::new(
                    workers,
                    budget.charged_memory_bytes(),
                    budget.work_ceiling(),
                )
            }),
    )
}

/// The placement this thread's test chose, or the World's.
pub(super) fn placed(
    world: RuntimeWorldExecutionPlacement<'_>,
) -> RuntimeWorldExecutionPlacement<'_> {
    let placement = match PLACEMENT.get() {
        WorthQueryExecutionPlacementForTest::World => return world,
        chosen => chosen,
    };
    let policy = match world {
        RuntimeWorldExecutionPlacement::Serial(policy)
        | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
    };
    let request_policy = |workers| match REQUEST_BUDGET.get().or(policy.accounting()) {
        Some(budget) => ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                workers,
                budget.charged_memory_bytes(),
                budget.work_ceiling(),
            ),
        ),
        None => ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            workers,
        ),
    };
    match placement {
        WorthQueryExecutionPlacementForTest::World => world,
        WorthQueryExecutionPlacementForTest::Serial => {
            RuntimeWorldExecutionPlacement::Serial(request_policy(NonZeroUsize::MIN))
        }
        WorthQueryExecutionPlacementForTest::Leased(workers)
        | WorthQueryExecutionPlacementForTest::Certified { workers, .. } => {
            RuntimeWorldExecutionPlacement::Leased {
                authority: query_test_execution_authority_for_test(),
                policy: request_policy(workers),
            }
        }
    }
}

/// The seed this thread's full reductions are certified under, if any.
pub(super) fn certifying() -> Option<u64> {
    match PLACEMENT.get() {
        WorthQueryExecutionPlacementForTest::Certified { seed, .. } => Some(seed),
        _ => None,
    }
}

/// Certifies a completed reduction: the serial oracle and the perturbed
/// backend of `seed` reduce `map` again on `lease`, outside the run's work
/// ceiling, and must agree with each other and with the run on the result's
/// bits and the charged work. A disagreement panics.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(super) fn certify<T, K, R, E, Kernel, Combine>(
    map: &ExecutionMap<T, K>,
    lease: &ExecutionResourceLease<'_>,
    seed: u64,
    kernel: Kernel,
    identity: R,
    combine: Combine,
    max_value_bytes: u64,
    (run, report): (&ReductionTree<R, Combine>, &ExecutionReport),
) -> Result<(), worth_execution::WorkCeilingDenial>
where
    T: Sync + ChargedBytes,
    R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
    E: Send + ChargedBytes,
    Kernel: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
    Combine: Fn(&R, &R) -> R + Clone + Sync,
{
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    let request_stop = |lease: &ExecutionResourceLease<'_>| {
        if lease.is_cancelled() {
            Some(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        } else if lease.deadline_elapsed() {
            Some(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
        } else {
            None
        }
    };
    let (checked, _) = worth_execution::ExecutionWorkCeiling::new(None).run_detached_for_test(
        lease,
        |diagnostic| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (certified, certified_report, _) = match map.certify_reduce(
                    diagnostic,
                    seed,
                    kernel,
                    identity,
                    combine,
                    max_value_bytes,
                    0,
                ) {
                    Ok(value) => value,
                    Err(failure) => {
                        return Err(certification_failure(diagnostic, seed, failure));
                    }
                };
                let same_bits =
                    match compare_canonical_values(diagnostic, run.result(), certified.result()) {
                        Ok(same) => same,
                        Err(stop) => {
                            if let Some(stop) = request_stop(diagnostic) {
                                return Err(stop);
                            }
                            panic!("the certified result could not be compared: {stop:?}");
                        }
                    };
                Ok((
                    same_bits,
                    certified_report.charged_work(),
                    certified_report.charged_span(),
                ))
            }))
        },
    )?;

    let (same_bits, work, span) = match checked {
        Ok(result) => result?,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    assert!(
        same_bits,
        "the run's result differs from the certified one under seed {seed}"
    );

    assert_eq!(
        report.charged_work(),
        work,
        "native canonical work differs under seed {seed}"
    );
    assert_eq!(
        report.charged_span(),
        span,
        "the run's charged span differs from the certified one under seed {seed}"
    );
    Ok(())
}

fn certification_failure<E>(
    lease: &ExecutionResourceLease<'_>,
    seed: u64,
    failure: ReduceCertificationFailure<E>,
) -> worth_execution::WorkCeilingDenial {
    use worth_execution::{
        MapKernelStop, MapStop, OracleMismatch, ReduceInputDenial, ReductionRunFailure,
        ReductionRunStop, WorkCeilingDenial,
    };
    match failure {
        ReduceCertificationFailure::Retention(worth_execution::RetainedMapDenial::Request(
            WorkCeilingDenial::Stopped(
                stop @ (MapKernelStop::Cancelled | MapKernelStop::DeadlineElapsed),
            ),
        )) => WorkCeilingDenial::Stopped(stop),
        ReduceCertificationFailure::Run(ReduceInputDenial::MapStopped {
            reason:
                MapStop::Failure {
                    cause:
                        MapKernelFailure::Stop(
                            stop @ (MapKernelStop::Cancelled | MapKernelStop::DeadlineElapsed),
                        ),
                    ..
                },
            ..
        })
        | ReduceCertificationFailure::Run(ReduceInputDenial::ReductionStopped {
            failure:
                ReductionRunFailure {
                    reason:
                        ReductionRunStop::Hook(
                            stop @ (MapKernelStop::Cancelled | MapKernelStop::DeadlineElapsed),
                        ),
                    ..
                },
            ..
        }) => WorkCeilingDenial::Stopped(stop),
        ReduceCertificationFailure::Mismatch(OracleMismatch::Stop) => {
            if lease.is_cancelled() {
                WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
            } else if lease.deadline_elapsed() {
                WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)
            } else {
                panic!("the oracle or perturbed run of seed {seed} stopped where the run did not")
            }
        }
        ReduceCertificationFailure::Mismatch(mismatch) => {
            panic!("the perturbed run of seed {seed} differs from the oracle: {mismatch:?}")
        }
        ReduceCertificationFailure::Run(_) | ReduceCertificationFailure::Retention(_) => {
            if lease.is_cancelled() {
                WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
            } else if lease.deadline_elapsed() {
                WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)
            } else {
                panic!("the oracle or perturbed run of seed {seed} stopped where the run did not")
            }
        }
    }
}

/// Bounds the test thread's requests with a host's declared budget.
/// Placement decides the backing; every request keeps its declared policy ceiling.
pub fn bound_advancement_requests_on_this_thread_for_test(
    budget: Option<ExecutionBudget>,
) -> Option<ExecutionBudget> {
    // Law 2, scope: every request-budget arm ends with its owned guard.
    let owner = owned_control_scope::current_id();
    if BUDGET_OWNER.get() != owner {
        let previous = (REQUEST_BUDGET.get(), BUDGET_OWNER.get());
        owned_control_scope::register(move || {
            if BUDGET_OWNER.get() == owner {
                let (budget, owner) = if owned_control_scope::is_live(previous.1) {
                    previous
                } else {
                    (None, 0)
                };
                REQUEST_BUDGET.set(budget);
                BUDGET_OWNER.set(owner);
            }
        });
    }
    BUDGET_OWNER.set(owner);
    REQUEST_BUDGET.replace(budget)
}

/// The host call uses exactly the computation fixture's declared budget.
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) struct FixturePlacement {
    _observation: crate::domain_computation::execution_observation::WorthQueryExecutionObservation,
    placement: WorthQueryExecutionPlacementForTest,
    budget: Option<worth_foundational::ExecutionBudget>,
}

#[cfg(test)]
impl FixturePlacement {
    pub(in crate::domain_computation::primary_graph) fn enter(
        placement: RuntimeWorldExecutionPlacement<'_>,
    ) -> Self {
        use self::{
            bound_advancement_requests_on_this_thread_for_test as bound,
            place_managed_computations_on_this_thread_for_test as place,
            WorthQueryExecutionPlacementForTest as Placement,
        };
        let observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
        let (chosen, policy) = match placement {
            RuntimeWorldExecutionPlacement::Serial(policy) => (Placement::Serial, policy),
            RuntimeWorldExecutionPlacement::Leased { policy, .. } => {
                (Placement::Leased(policy.max_workers()), policy)
            }
        };
        let prior = place(chosen);
        if prior != Placement::World {
            place(prior);
        }
        Self {
            _observation: observation,
            placement: prior,
            budget: bound(policy.accounting()),
        }
    }
}

#[cfg(test)]
impl Drop for FixturePlacement {
    fn drop(&mut self) {
        use self::{
            bound_advancement_requests_on_this_thread_for_test as bound,
            place_managed_computations_on_this_thread_for_test as place,
        };
        bound(self.budget);
        place(self.placement);
    }
}

#[cfg(test)]
mod scope_tests;

#[cfg(test)]
mod certification_tests;
