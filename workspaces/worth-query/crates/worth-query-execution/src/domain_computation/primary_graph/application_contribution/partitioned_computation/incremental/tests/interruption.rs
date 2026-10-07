//! A request cancelled while the reducer combines stops at the next node of
//! the reduction tree, on a full run and on an incremental one, and its
//! interruption is never retried as a rebuild.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::super::WorthQueryManagedComputationInterruption;
use super::*;

thread_local! {
    static COMBINES: Cell<usize> = const { Cell::new(0) };
    /// The request the next combine cancels, if one is armed.
    static ARMED: RefCell<Option<WorthQueryCancellationSource>> = const { RefCell::new(None) };
}

/// Sums, and cancels the armed request inside the combine it is met in.
fn cancelling_sum(left: &u64, right: &u64) -> u64 {
    COMBINES.set(COMBINES.get() + 1);
    if let Some(source) = ARMED.take() {
        source.cancel();
    }
    left + right
}

/// An attempt whose request its first combine cancels, and how many
/// combines ran.
fn cancelled_in_reducer(
    world: &AuthorizationWorld,
    installed: &Installed,
    prior: Option<ComputationPrior>,
) -> (Attempt, usize) {
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token());
    COMBINES.set(0);
    ARMED.set(Some(source));
    let attempt = attempt_in(world, installed, prior, &request);
    assert!(ARMED.take().is_none(), "the reducer met the armed request");
    (attempt, COMBINES.get())
}

fn cancelled() -> Outcome {
    Err(WorthQueryPartitionedComputationDenial::Interrupted(
        WorthQueryManagedComputationInterruption::Cancelled,
    ))
}

/// Four partitions build four tree nodes of two combines each. Cancelled
/// inside the first combine, the run stops before the second.
#[test]
fn a_full_run_cancelled_inside_a_combine_stops_at_the_next_tree_node() {
    let world = installed_authorization_world(true);
    let installed = installed_over(4, StatusRead::Gather(1), cancelling_sum);
    COMBINES.set(0);
    let whole = attempt(&world, &installed, None);
    assert_eq!(whole.outcome.unwrap().0, 1 + 2 + 3 + 4);
    assert_eq!(COMBINES.get(), 8, "four nodes take two combines each");

    let (stopped, combines) = cancelled_in_reducer(&world, &installed, None);
    assert!(
        stopped.runs.is_empty(),
        "an interrupted run never completes"
    );
    assert_eq!(stopped.gathered, [0, 1, 2, 3]);
    assert_eq!(stopped.outcome, cancelled());
    assert_eq!(combines, 1, "no combine runs after the cancelling one");
    let [(partitions, tree)] = stopped.tree_runs.as_slice() else {
        panic!("one stopped tree report")
    };
    assert!(matches!(partitions, Run::Full(_)));
    assert_eq!(tree.metrics().combine_calls, 1);
    assert_eq!(tree.metrics().recombined_nodes, 0);
}

/// A moved fact recombines its leaf's path, two combines a node. Cancelled
/// inside the first, the run stops before the second and does not rebuild
/// the tree.
#[test]
fn an_incremental_run_cancelled_inside_a_recombine_stops_and_does_not_rebuild() {
    let world = installed_authorization_world(true);
    let installed = installed_over(4, StatusRead::Gather(1), cancelling_sum);
    let first = attempt(
        &world,
        &installed,
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
    );
    assert_eq!(first.gathered, [0, 1, 2, 3]);
    *installed.owner.bump.lock().unwrap() = 10;

    let (stopped, combines) = cancelled_in_reducer(&world, &installed, Some(prior_of(first, true)));
    assert!(
        stopped.runs.is_empty(),
        "an interrupted run never completes"
    );
    assert_eq!(
        stopped.gathered,
        [1],
        "the run carried the other partitions"
    );
    assert_eq!(stopped.outcome, cancelled());
    assert_eq!(combines, 1, "neither a second recombine nor a rebuild ran");
    let [(partitions, tree)] = stopped.tree_runs.as_slice() else {
        panic!("one stopped tree report")
    };
    assert_eq!(*partitions, Run::Incremental);
    assert!(matches!(
        tree,
        super::super::tree_report::WorthQueryPartitionedTreeRun::Edited(_)
    ));
    assert_eq!(tree.metrics().combine_calls, 1);
    assert_eq!(tree.metrics().recombined_nodes, 0);
}
