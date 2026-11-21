//! C3.3: a conditional operation whose commit answer was lost (R9
//! `OperationIndeterminate`) is killed. Reopen classifies its durable row by
//! World's state before service, and the retry of the original intent yields
//! exactly one occurrence: published, recovered unpublished, or no effect.
use super::*;
use super::temporal_home::{applies, installed_commits, kill_at_world_compare, revision};
use crate::host::application_home::temporal::{observe, open};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::primary_graph;

/// World's state for the lost answer at the kill.
#[derive(Clone, Copy)]
enum Cut {
    /// The commit published; only its answer was lost.
    Published,
    /// Nothing committed; a retry then reached World's compare with its
    /// Native effect durable and the process died there.
    RecoveredUnpublished,
    /// Nothing committed.
    NoEffect,
}

#[test]
fn indeterminate_after_world_published_finishes_with_the_original_after_kill() {
    journey(
        Cut::Published,
        "indeterminate_after_world_published_finishes_with_the_original_after_kill",
    );
}
#[test]
fn indeterminate_with_its_original_unpublished_continues_that_original_after_kill() {
    journey(
        Cut::RecoveredUnpublished,
        "indeterminate_with_its_original_unpublished_continues_that_original_after_kill",
    );
}
#[test]
fn indeterminate_without_a_committed_open_executes_fresh_once_after_kill() {
    journey(
        Cut::NoEffect,
        "indeterminate_without_a_committed_open_executes_fresh_once_after_kill",
    );
}

fn journey(cut: Cut, behavior: &str) {
    child_reexecution::drive(
        &format!("conditional_indeterminate::{behavior}"),
        || writer(cut),
        || reader(cut),
    );
}

fn writer(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let commits = installed_commits(&home);
    let runtime = home.runtime().runtime();
    let rejection = match cut {
        Cut::Published => None,
        Cut::RecoveredUnpublished | Cut::NoEffect => Some(
            runtime
                .reject_next_commit_before_transaction_for_test()
                .unwrap(),
        ),
    };
    runtime.lose_next_commit_answer_for_test();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe(&home, &execution)
    else {
        panic!("the due intent executes");
    };
    assert_eq!(receipt.indeterminate_operation_count(), 1);
    drop((receipt, rejection));
    let [row] = home
        .runtime()
        .conditional_decision_obligations()
        .try_into()
        .unwrap();
    assert_eq!(
        row.posture,
        primary_graph::QueryConditionalPosture::Indeterminate(
            primary_graph::QueryConditionalOriginal::Unclassified
        ),
        "the live owner records the lost answer before custody leaves it"
    );
    assert_eq!(applies(&home), 1);
    let published = matches!(cut, Cut::Published);
    assert_eq!(revision(&home), if published { 2 } else { 1 });
    assert_eq!(installed_commits(&home), commits + usize::from(published));
    if let Cut::RecoveredUnpublished = cut {
        // The retry of the original intent reaches World's compare.
        kill_at_world_compare(&home, 2);
    }
    child_reexecution::wait_for_kill();
}

fn reader(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let rows = home.runtime().conditional_decision_obligations();
    match cut {
        Cut::Published | Cut::NoEffect => assert!(
            rows.is_empty(),
            "before service the row finished with World's answer: {rows:?}"
        ),
        Cut::RecoveredUnpublished => {
            let [row] = rows.as_slice() else {
                panic!("the obligation is retained: {rows:?}")
            };
            assert!(
                matches!(
                    row.posture,
                    primary_graph::QueryConditionalPosture::Indeterminate(
                        primary_graph::QueryConditionalOriginal::Recovered(_)
                    )
                ),
                "joined to its recovered original: {row:?}"
            );
        }
    }
    let commits = installed_commits(&home);
    let published = matches!(cut, Cut::Published);
    assert_eq!(revision(&home), if published { 2 } else { 1 });
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe(&home, &execution)
    else {
        panic!("the reopened clock observes");
    };
    let (fresh, original) = match cut {
        Cut::Published => (0, 0),
        Cut::RecoveredUnpublished => (0, 1),
        Cut::NoEffect => (1, 0),
    };
    assert_eq!(receipt.committed_operation_count(), fresh);
    assert_eq!(receipt.already_committed_operation_count(), original);
    drop(receipt);
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    assert_eq!(
        applies(&home),
        fresh,
        "only a proven no-effect key runs the operation again"
    );
    assert_eq!(revision(&home), 2);
    assert_eq!(
        installed_commits(&home),
        commits + usize::from(!published),
        "exactly one original occurrence"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the settled Home closes: {refused:?}"))
        .reclaim();
    let home = open(&child_reexecution::location(), &execution);
    assert!(home.runtime().conditional_decision_obligations().is_empty());
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
