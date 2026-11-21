//! C3.3 / Phase 7: a temporal operation killed between its commit and its
//! response. Its idempotency key derives only from durable Home identities, so
//! re-execution after reopen resolves to the exact original occurrence, once.
use super::temporal_home::{
    applies, head_commit_projection, installed_commits, kill_at_world_compare,
    published_occurrence_projection, revision,
};
use super::*;
use crate::host::application_home::temporal::{observe, open, retry_original_intent_on_branch};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::primary_graph;

/// Where the response was lost before reaching the caller.
#[derive(Clone, Copy)]
enum Cut {
    /// The Native commit is durable; World has not moved the product.
    BeforeProductMovement,
    /// World published the occurrence; the worker is parked before returning
    /// the answer to the caller of `observe`.
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
            let pause = home
                .runtime()
                .runtime()
                .pause_after_commit_before_answer_for_test();
            std::thread::scope(|threads| {
                threads.spawn(|| {
                    let lease = checkpoint_capture::lease(128 << 20);
                    drop(observe(
                        &home,
                        &worth_execution::ExecutionRequest::leased(&lease),
                    ));
                });
                assert!(
                    pause.wait_until_reached(Duration::from_secs(20)),
                    "World published the temporal commit and parked before returning to observe()"
                );
                assert_eq!(applies(&home), 1);
                assert_eq!(revision(&home), 2);
                let branch = home.runtime().current_world();
                std::fs::write(
                    child_reexecution::expected(),
                    head_commit_projection(&home, branch),
                )
                .unwrap();
                child_reexecution::wait_for_kill();
            });
        }
    }
}

fn reader(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let branch = home.runtime().current_world();
    let commits = installed_commits(&home);
    let moved = matches!(cut, Cut::AfterPublication);
    assert_eq!(revision(&home), if moved { 2 } else { 1 });

    // For BeforeProductMovement, capture World's exact recovered attempt
    // identity before re-execution continues it.
    let recovered_attempt = if !moved {
        let page = home
            .runtime()
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(32).unwrap())
            .unwrap();
        let [row] = page.rows() else {
            panic!("expected one recovered World original");
        };
        let claim = home
            .runtime()
            .runtime()
            .readmit_product_publication_recovery(row.handle())
            .unwrap();
        let effects = claim.inspect().unwrap();
        let attempt = effects.attempt_identity();
        let durable = worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
            attempt.owner_identity().durable().unwrap(),
            attempt.ordinal(),
        );
        drop(effects);
        drop(claim);
        Some(durable)
    } else {
        None
    };

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
    let observed_commit = receipt
        .execution_provenance()
        .first()
        .and_then(|p| p.published_commit().cloned());
    drop(receipt);

    let primary_graph::WorthQueryConditionalOriginalResolution::Published(original) =
        retry_original_intent_on_branch(&home, branch, &execution)
    else {
        panic!("re-executing the original temporal intent returns its published occurrence");
    };
    if moved {
        let expected = std::fs::read_to_string(child_reexecution::expected()).unwrap();
        assert_eq!(
            published_occurrence_projection(&original),
            expected,
            "retry after kill returns the exact original published occurrence"
        );
    } else {
        let published_commit = original
            .new_product_head()
            .durable()
            .unwrap()
            .selected_commit();
        assert_eq!(observed_commit.as_ref(), Some(&published_commit));
        assert_eq!(original.attempt_identity().durable(), recovered_attempt);
    }
    drop(original);

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
