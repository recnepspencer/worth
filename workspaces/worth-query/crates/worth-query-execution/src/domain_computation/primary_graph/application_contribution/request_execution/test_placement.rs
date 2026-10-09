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

thread_local! {
    static REQUEST_BUDGET: Cell<Option<ExecutionBudget>> = const { Cell::new(None) };
    static PLACEMENT: Cell<WorthQueryExecutionPlacementForTest> =
        const { Cell::new(WorthQueryExecutionPlacementForTest::World) };
}

/// Places the managed computations of the requests this thread opens from
/// now on, and returns the placement it replaces.
pub fn place_managed_computations_on_this_thread_for_test(
    placement: WorthQueryExecutionPlacementForTest,
) -> WorthQueryExecutionPlacementForTest {
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
            charged_memory_bytes: 1 << 40,
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
    // A whole host integration binary can certify its declared policy on a
    // lease, including worker threads; explicit per-thread placement wins.
    let placement = match PLACEMENT.get() {
        WorthQueryExecutionPlacementForTest::World => {
            std::env::var("WORTH_QUERY_TEST_REQUEST_WORKERS")
                .ok()
                .map(|value| {
                    WorthQueryExecutionPlacementForTest::Leased(
                        value
                            .parse::<usize>()
                            .ok()
                            .and_then(NonZeroUsize::new)
                            .expect("test request workers is a positive width"),
                    )
                })
                .unwrap_or(WorthQueryExecutionPlacementForTest::World)
        }
        chosen => chosen,
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
    REQUEST_BUDGET.replace(budget)
}
