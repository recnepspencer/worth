//! C3.3 / Phase 7: a temporal operation killed between its commit and its
//! response. Its idempotency key derives only from durable Home identities, so
//! re-execution after reopen resolves to the original occurrence, once.
use super::temporal_home::{applies, installed_commits, kill_at_world_compare, revision};
use super::*;
use crate::host::application_home::temporal::{observe, open};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::primary_graph;

/// Where the response was lost.
#[derive(Clone, Copy)]
enum Cut {
    /// The Native commit is durable; World has not moved the product.
    BeforeProductMovement,
    /// World published the occurrence; the response never reached the caller.
    AfterPublication,
}

#[test]
fn temporal_commit_killed_before_product_movement_reexecutes_to_its_original() {
    journey(
        Cut::BeforeProductMovement,
        "temporal_commit_killed_before_product_movement_reexecutes_to_its_original",
    );
}
#[test]
fn temporal_commit_killed_after_publication_reexecutes_nothing() {
    journey(
        Cut::AfterPublication,
        "temporal_commit_killed_after_publication_reexecutes_nothing",
    );
}

fn journey(cut: Cut, behavior: &str) {
    child_reexecution::drive(
        &format!("temporal_reexecution::{behavior}"),
        || writer(cut),
        || reader(cut),
    );
}

fn writer(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    match cut {
        Cut::BeforeProductMovement => kill_at_world_compare(&home, 1),
        Cut::AfterPublication => {
            // Nothing durable is written between World's publication and the
            // response: a kill after it leaves the same durable state.
            drop(observe(&home, &execution));
            assert_eq!(applies(&home), 1);
            assert_eq!(revision(&home), 2);
            child_reexecution::wait_for_kill();
        }
    }
}

fn reader(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let commits = installed_commits(&home);
    let moved = matches!(cut, Cut::AfterPublication);
    assert_eq!(revision(&home), if moved { 2 } else { 1 });
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe(&home, &execution)
    else {
        panic!("the reopened clock observes");
    };
    assert_eq!(
        receipt.committed_operation_count(),
        0,
        "never a fresh original"
    );
    assert_eq!(
        receipt.already_committed_operation_count(),
        usize::from(!moved),
        "re-execution resolves to the original occurrence"
    );
    drop(receipt);
    assert_eq!(applies(&home), 0, "the operation never runs again");
    assert_eq!(revision(&home), 2);
    assert_eq!(
        installed_commits(&home),
        commits + usize::from(!moved),
        "the recovered original publishes once; nothing else does"
    );
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the re-executed Home closes: {refused:?}"))
        .reclaim();
    let home = open(&child_reexecution::location(), &execution);
    assert_eq!(revision(&home), 2);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe(&home, &execution)
    else {
        panic!("the reopened clock observes");
    };
    assert_eq!(receipt.committed_operation_count(), 0);
    drop(receipt);
    assert_eq!(applies(&home), 0);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the reopened Home closes: {refused:?}"))
        .reclaim();
}
