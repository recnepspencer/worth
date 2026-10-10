//! A test's choice of where the requests its thread opens run their managed
//! computations, in place of the World's placement: serially, on a lease of
//! a chosen worker count, or on such a lease with each full reduction also
//! certified against the serial oracle and a seeded perturbed backend.
//!
//! Every lease is drawn from the one authority a process may construct. It
//! admits twice the machine's width, so a lease may ask for more workers than
//! the machine has. Each policy carries its own budget, far under the
//! authority's, so tests running at once never meet on its memory.

use std::cell::Cell;
use std::num::NonZeroUsize;
use std::sync::OnceLock;

use worth_execution::{
    compare_canonical_values, CanonicalBits, ChargedBytes, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionMap, ExecutionResourceLease, MapKernelContext,
    MapKernelFailure, ReduceCertificationFailure, ReductionTree,
};
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

/// The most workers a test lease may ask for: twice the machine's width.
pub fn test_execution_workers() -> NonZeroUsize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .saturating_mul(NonZeroUsize::new(2).expect("two is not zero"))
}

/// This process's one authority, admitting [`test_execution_workers`].
pub(in crate::domain_computation::primary_graph) fn test_authority() -> &'static ExecutionAuthority
{
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: test_execution_workers(),
            charged_memory_bytes: Some(1 << 40),
        })
        .expect("a test process constructs one authority, here")
    })
}

/// A canonical, automatic policy of `workers` and `memory`, with no work
/// limit beyond each computation's own.
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn test_policy(
    workers: NonZeroUsize,
    memory: u64,
) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        REQUEST_BUDGET
            .get()
            .map_or(ExecutionBudget::new(workers, memory, u64::MAX), |budget| {
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
    let request_policy = |workers| {
        let budget = REQUEST_BUDGET.get().unwrap_or(policy.budget());
        ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                workers,
                budget.charged_memory_bytes(),
                budget.work_ceiling(),
            ),
        )
    };
    match placement {
        WorthQueryExecutionPlacementForTest::World => world,
        WorthQueryExecutionPlacementForTest::Serial => {
            RuntimeWorldExecutionPlacement::Serial(request_policy(NonZeroUsize::MIN))
        }
        WorthQueryExecutionPlacementForTest::Leased(workers)
        | WorthQueryExecutionPlacementForTest::Certified { workers, .. } => {
            RuntimeWorldExecutionPlacement::Leased {
                authority: test_authority(),
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
pub(super) fn certify<T, K, R, E, Kernel, Combine>(
    map: &ExecutionMap<T, K>,
    lease: &ExecutionResourceLease<'_>,
    seed: u64,
    kernel: Kernel,
    identity: R,
    combine: Combine,
    max_value_bytes: u64,
    (run, report): (&ReductionTree<R, Combine>, &ExecutionReport),
) where
    T: Sync + ChargedBytes,
    R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
    E: Send + ChargedBytes,
    Kernel: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
    Combine: Fn(&R, &R) -> R + Clone + Sync,
{
    let (certified, certified_report, _) = map
        .certify_reduce(lease, seed, kernel, identity, combine, max_value_bytes, 0)
        .unwrap_or_else(|failure| match failure {
            ReduceCertificationFailure::Run(_) => {
                panic!(
                    "the oracle or the perturbed run of seed {seed} stopped where the run did not"
                )
            }
            ReduceCertificationFailure::Mismatch(mismatch) => {
                panic!("the perturbed run of seed {seed} differs from the oracle: {mismatch:?}")
            }
        });
    let same_bits = compare_canonical_values(lease, run.result(), certified.result())
        .unwrap_or_else(|stop| panic!("the certified result could not be compared: {stop:?}"));
    assert!(
        same_bits,
        "the run's result differs from the certified one under seed {seed}"
    );
    assert_eq!(
        report.charged_work(),
        certified_report.charged_work(),
        "the run's charged work differs from the certified one under seed {seed}"
    );
}

/// Bounds the test thread's requests with a host's declared budget.
/// Placement still decides the backing; a serial request has no policy work ceiling.
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

#[cfg(test)]
mod scope_tests {
    use super::*;
    use crate::domain_computation::execution_observation::WorthQueryExecutionObservation as Observation;
    #[test]
    fn unwinding_clears_placement_and_budget_before_the_thread_is_reused() {
        assert!(std::panic::catch_unwind(|| {
            let _guard = Observation::install_on_this_thread();
            place_managed_computations_on_this_thread_for_test(
                WorthQueryExecutionPlacementForTest::Serial,
            );
            bound_advancement_requests_on_this_thread_for_test(Some(ExecutionBudget::new(
                NonZeroUsize::MIN,
                7,
                9,
            )));
            panic!("exercise placement cleanup");
        })
        .is_err());
        assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
        assert!(REQUEST_BUDGET.get().is_none());
    }
    #[test]
    fn dropping_an_older_guard_cannot_clear_a_newer_guards_placement() {
        let outer = Observation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Serial,
        );
        let inner = Observation::install_on_this_thread();
        let leased = WorthQueryExecutionPlacementForTest::Leased(NonZeroUsize::MIN);
        place_managed_computations_on_this_thread_for_test(leased);
        drop(outer);
        assert_eq!(PLACEMENT.get(), leased);
        drop(inner);
        assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
    }
    #[test]
    fn an_inner_guard_restores_only_a_still_owned_outer_placement() {
        let outer = Observation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Serial,
        );
        let inner = Observation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Leased(NonZeroUsize::MIN),
        );
        drop(inner);
        assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::Serial);
        drop(outer);
        assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
    }
}
