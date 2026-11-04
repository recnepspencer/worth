//! A request cancelled while the reducer combines stops at the next node of
//! the reduction tree, on a full run and on an incremental one, and its
//! interruption is never retried as a rebuild.

use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;

thread_local! {
    pub static COMBINES: Cell<usize> = const { Cell::new(0) };
    /// The request the next combine cancels, if one is armed.
    pub static ARMED: RefCell<Option<WorthQueryCancellationSource>> = const { RefCell::new(None) };
}

/// Sums, and cancels the armed request inside the combine it is met in.
pub fn cancelling_sum(left: &u64, right: &u64) -> u64 {
    COMBINES.set(COMBINES.get() + 1);
    if let Some(source) = ARMED.take() {
        source.cancel();
    }
    left + right
}

/// An attempt whose request its first combine cancels, and how many
/// combines ran.
pub fn cancelled_in_reducer(
    world: &AuthorizationWorld,
    installed: &Installed,
    prior: Option<ComputationPrior>,
) -> (Attempt, usize) {
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    COMBINES.set(0);
    ARMED.set(Some(source));
    let attempt = attempt_in(world, installed, prior, &request);
    assert!(ARMED.take().is_none(), "the reducer met the armed request");
    (attempt, COMBINES.get())
}

pub fn cancelled() -> Outcome {
    Err(WorthQueryPartitionedComputationDenial::Interrupted(
        WorthQueryManagedComputationInterruption::Cancelled,
    ))
}

/// Four partitions build four tree nodes of two combines each. Cancelled
/// inside the first combine, the run stops before the second.
#[test]
fn a_full_run_cancelled_inside_a_combine_stops_at_the_next_tree_node() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
    let [tree] = stopped.tree_runs.as_slice() else {
        panic!("one stopped tree report")
    };
    assert!(matches!(
        tree,
        worth_query_execution::facade::application_contribution::WorthQueryPartitionedTreeRun::Full(
            Cause::NoProducerPrior,
            _
        )
    ));
    assert_eq!(
        tree.metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls,
        1
    );
    assert_eq!(
        tree.metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .recombined_nodes,
        0
    );
}

/// A moved fact recombines its leaf's path, two combines a node. Cancelled
/// inside the first, the run stops before the second and does not rebuild
/// the tree.
#[test]
fn an_incremental_run_cancelled_inside_a_recombine_stops_and_does_not_rebuild() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed_over(4, StatusRead::Gather(1), cancelling_sum);
    let first = attempt(
        &world,
        &installed,
        Some(ComputationPrior::new_for_test(
            edition(),
            Err(Cause::FirstRun),
            None,
        )),
    );
    assert_eq!(first.gathered, [0, 1, 2, 3]);
    *(*installed.owner_for_test()).bump.lock().unwrap() = 10;

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
    let [tree] = stopped.tree_runs.as_slice() else {
        panic!("one stopped tree report")
    };
    assert_eq!(tree.partitions_for_test(), Run::Incremental);
    assert!(matches!(
        tree,
        worth_query_execution::facade::application_contribution::WorthQueryPartitionedTreeRun::Edited(_)
    ));
    assert_eq!(
        tree.metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls,
        1
    );
    assert_eq!(
        tree.metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .recombined_nodes,
        0
    );
}
