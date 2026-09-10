//! The creation path rechecks the product head between its two owner forks.
//!
//! Reaching that recheck with a displaced head needs another operation to
//! settle in the window after the first owner leg and before the recheck, which
//! no budget or intent can arrange on its own. The rehearsal seam is keyed to
//! this owner, armed for exactly one creation, and bounded, so a creation that
//! is never released fails by name instead of hanging.

use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::*;

use crate::publication::{RelationalAttemptProgressPosture, SignalAttemptProgressPosture};

const STALE_HEAD_TEST_TIMEOUT: Duration = Duration::from_secs(5);
const STALE_HEAD_RELATIONAL_TARGET: &str = "relational-branch-stale-head";

/// Run one creation, hold it at its between-owners boundary, let the caller act
/// on the world while it is held, then release it and return where it landed.
fn creation_interrupted_at_the_owner_boundary(
    owner: &Arc<TestOwner>,
    source: ProductBranchObservation,
    intent: ProductBranchCreationIntent,
    while_held: impl FnOnce(&TestOwner),
) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
    let (reached_tx, reached_rx) = mpsc::sync_channel(1);
    let rehearsal = owner.rehearse_creation_fork_boundary(reached_tx);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    let worker_owner = Arc::clone(owner);
    let worker = std::thread::spawn(move || {
        let cancellation = RuntimeWorldCancellationSource::new();
        let outcome = RuntimeWorldBranchService::create_product_branch(
            worker_owner.as_ref(),
            RuntimeWorldBranchCreationRequest::new(source, intent, &cancellation.token()),
        );
        finished_tx
            .send(outcome)
            .expect("the boundary proof still owns its completion receiver");
    });

    let paused = reached_rx
        .recv_timeout(STALE_HEAD_TEST_TIMEOUT)
        .expect("the creation reaches its between-owners boundary");
    assert_eq!(paused, owner.owner_identity());
    while_held(owner.as_ref());

    drop(rehearsal);
    let outcome = finished_rx
        .recv_timeout(STALE_HEAD_TEST_TIMEOUT)
        .expect("the creation finishes once its boundary is released");
    worker.join().expect("the creation worker does not panic");
    outcome
}

#[test]
fn stale_product_head_after_fork_before_advance_retains_fork_and_winner_evidence() {
    let (mut fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let intent = fork_intent(
        "branch-stale-head",
        relational_fork(STALE_HEAD_RELATIONAL_TARGET),
        SignalBranchCreationPlan::ReuseExact,
    );
    let held_source = source.clone();
    let outcome = creation_interrupted_at_the_owner_boundary(
        &owner,
        source.clone(),
        intent,
        |paused_owner| {
            assert_eq!(
                paused_owner.state.custody.installed(),
                1,
                "the Relational fork really happened before the boundary"
            );
            // A different operation wins the product head this creation was
            // admitted against, while the creation is held between its owners.
            seed_relational_source(paused_owner, &mut fixture, held_source);
        },
    )
    .expect("a displaced product head is a product-unpublished outcome");

    let winner = current_root_observation(owner.as_ref());
    assert_ne!(winner.selected_commit(), source.selected_commit());
    let effects = match outcome {
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => effects,
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a creation whose product head moved cannot install its destination")
        }
    };
    assert_eq!(effects.cause(), ProductUnpublishedCause::StaleProductHead);
    assert_eq!(
        effects.progress().relational_posture(),
        RelationalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_winner_and_custody_are_retained(owner.as_ref(), &effects, &winner);
    drop(effects);
    drop(fixture);
}

/// The retained record must name the occurrence that displaced this creation
/// and keep the component branch the creation really made.
fn assert_winner_and_custody_are_retained(
    owner: &TestOwner,
    effects: &crate::recovery::ProductUnpublishedOwnerEffects,
    winner: &ProductBranchObservation,
) {
    let observed = effects
        .last_observed_head()
        .expect("a displaced creation retains the head that displaced it");
    assert_eq!(observed.commit().identity(), winner.selected_commit());
    assert_eq!(observed.branch(), winner.branch_identity());

    let records = owner.state.custody.installed_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target().name(), STALE_HEAD_RELATIONAL_TARGET);
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
}

/// The same boundary, reached before either owner has moved: there is no effect
/// to retain, so the creation is a plain denial that names the displaced source
/// head rather than a vague owner unavailability.
#[test]
fn stale_product_head_before_any_owner_effect_denies_as_a_stale_source_head() {
    let (mut fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let intent = fork_intent(
        "branch-stale-source-head",
        RelationalBranchCreationPlan::ReuseExact,
        signal_fork("signal-branch-stale-source-head"),
    );
    let held_source = source.clone();
    let denial =
        creation_interrupted_at_the_owner_boundary(&owner, source, intent, |paused_owner| {
            assert_eq!(
                paused_owner.state.custody.installed(),
                0,
                "an exactly reused Relational component performs nothing before the boundary"
            );
            seed_relational_source(paused_owner, &mut fixture, held_source);
        })
        .expect_err("a displaced source head denies a creation that moved nothing");

    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
    assert_eq!(owner.state.custody.installed(), 0);
    assert_eq!(
        owner.recovery_record_count(),
        0,
        "a creation with no owner effect leaves nothing to recover"
    );
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    drop(fixture);
}
