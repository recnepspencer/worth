//! An incremental run holds its next tree's declared bound before it builds
//! it. A run whose path combines no longer fit its work rebuilds the tree
//! from every leaf, as a full run builds it, and a budget one byte short of
//! that build refuses it before the reducer combines anything.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::ReductionTree;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use super::super::super::super::request_execution::{test_authority, test_policy};
use super::super::super::super::WorthQueryManagedComputationResourceDenial as Resource;
use super::*;

static COMBINES: AtomicUsize = AtomicUsize::new(0);

fn counted_sum(left: &u64, right: &u64) -> u64 {
    COMBINES.fetch_add(1, Ordering::Relaxed);
    left + right
}

/// The declared work and result bytes of the test computation.
const DECLARED: u64 = 4096;

/// A second run of four partitions whose one recomputed kernel leaves one
/// unit of work short of the combines the retained tree charges, so the tree
/// is built again. Placed under `memory`, leased, then serial, each
/// with the combines it ran.
fn rebuilt_under(memory: Option<u64>) -> Vec<(Outcome, usize)> {
    let placements: Vec<Box<dyn Fn() -> RuntimeWorldExecutionPlacement<'static>>> = match memory {
        None => vec![Box::new(|| {
            RuntimeWorldExecutionPlacement::Serial(
        crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
            std::num::NonZeroUsize::MIN, 1 << 30,
        ),
    )
        })],
        Some(memory) => {
            let policy = move || test_policy(NonZeroUsize::new(2).unwrap(), memory);
            vec![
                Box::new(move || RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy: policy(),
                }),
                Box::new(move || RuntimeWorldExecutionPlacement::Serial(policy())),
            ]
        }
    };
    let world = installed_authorization_world(true);
    placements
        .iter()
        .map(|placement| {
            let installed = installed_over(4, StatusRead::Gather(1), counted_sum);
            let first = first_run_over_four(&world, &installed);
            // The first run charged its calls, four kernels of one unit and
            // its combines. The moved kernel takes the place of one unit and
            // all but one combine, so one combine short of the tree is left.
            let charged = first.outcome.as_ref().unwrap().1;
            let moved = DECLARED + 2 - charged;
            *installed.owner.bump.lock().unwrap() = 10;
            *installed.owner.work.lock().unwrap() = [usize::try_from(moved).unwrap(), 1];
            COMBINES.store(0, Ordering::Relaxed);
            let run = attempt_placed(
                &world,
                &installed,
                Some(prior_of(first, true)),
                &live_scope(),
                placement(),
            );
            assert_eq!(run.gathered, [1], "only the moved partition gathers again");
            (run.outcome, COMBINES.load(Ordering::Relaxed))
        })
        .collect()
}

fn first_run_over_four(world: &AuthorizationWorld, installed: &Installed) -> Attempt {
    let first = attempt(
        world,
        installed,
        Some(ComputationPrior::new(
            edition(),
            Err(Cause::NoPriorRecord),
            None,
        )),
    );
    assert_eq!(first.gathered, [0, 1, 2, 3]);
    first
}

fn refused_by_memory(outcome: &Outcome) -> Option<u64> {
    match outcome {
        Err(WorthQueryPartitionedComputationDenial::Resource(Resource::MemoryLimit {
            requested,
            ..
        })) => Some(*requested),
        _ => None,
    }
}

#[test]
fn a_rebuild_one_byte_short_of_its_bound_refuses_before_it_combines() {
    let unbounded = rebuilt_under(None);
    let [(outcome, reached)] = unbounded.as_slice() else {
        panic!("one unbounded placement");
    };
    let reached = *reached;
    assert_eq!(
        *outcome,
        Err(WorthQueryPartitionedComputationDenial::Resource(
            Resource::WorkExhausted
        )),
        "the rebuild stops where a full run's build stops"
    );
    assert!(reached > 0, "it combined as far as its work reached");

    let admits = |memory| {
        rebuilt_under(Some(memory))
            .iter()
            .all(|(outcome, _)| refused_by_memory(outcome).is_none())
    };
    let (mut refused, mut admitted) = (0_u64, 1 << 24);
    assert!(admits(admitted));
    while admitted - refused > 1 {
        let middle = refused + (admitted - refused) / 2;
        if admits(middle) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    let build =
        ReductionTree::<u64, fn(&u64, &u64) -> u64>::checked_build_memory_bound(4, DECLARED)
            .unwrap();
    for (outcome, combines) in rebuilt_under(Some(refused)) {
        let requested = refused_by_memory(&outcome).expect("one byte short is a memory refusal");
        assert!(
            requested > build,
            "the refusal is the rebuild's own bound: {requested} against a build of {build}"
        );
        assert_eq!(combines, 0, "nothing is combined before the bound is held");
    }
    for (_, combines) in rebuilt_under(Some(admitted)) {
        assert_eq!(combines, reached, "with its bound held the rebuild runs");
    }
}
