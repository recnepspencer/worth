//! Installation under the source head's guard.
//!
//! Exact reuse and fork finalization both recheck the source head before they
//! charge anything, and a publication or retirement can land between that
//! recheck and the registry installation. The installation therefore happens
//! under the source cell's own read guard, so the recheck and the install are
//! one step: a source that moved in the window is refused at the install,
//! named the way the pre-effect recheck names it, and nothing is installed
//! from a head that is no longer current. The window is reached through the
//! seam that holds one creation just before it takes the guard.
//!
//! A source branch that holds no occurrence at all is `RetiredBranch` on
//! every creation route, the same answer observation gives for that branch,
//! whether it was retired before the creation was admitted or while the
//! creation was held at the guard.

use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::fork_creation::{
    current_root_observation, fork_intent, relational_fork, seed_relational_source,
    setup_with_relational_source, signal_fork,
};
use super::{create_reused_branch, owner_lifecycles, reuse_intent, TestOwner};
use crate::branch::{
    ProductBranchCreationIntent, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial,
};
use crate::lifecycle::{
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest,
    RuntimeWorldBranchService, RuntimeWorldObservationService,
};
use crate::publication::{
    RelationalAttemptProgressPosture, RuntimeWorldCancellationSource, SignalAttemptProgressPosture,
};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};

const SOURCE_GUARD_TEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Run one creation, hold it just before its source-guarded installation, let
/// the caller act on the world while it is held, then release it and return
/// where it landed. A creation that panics while held surfaces its own panic,
/// not the completion channel's disconnection.
fn creation_interrupted_before_install(
    owner: &Arc<TestOwner>,
    source: ProductBranchObservation,
    intent: ProductBranchCreationIntent,
    while_held: impl FnOnce(&TestOwner),
) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
    let (reached_tx, reached_rx) = mpsc::sync_channel(1);
    let rehearsal = owner.rehearse_source_guarded_install(reached_tx);
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
            .expect("the guard proof still owns its completion receiver");
    });

    let paused = reached_rx
        .recv_timeout(SOURCE_GUARD_TEST_TIMEOUT)
        .expect("the creation reaches its source-guarded installation");
    assert_eq!(paused, owner.owner_identity());
    while_held(owner.as_ref());

    drop(rehearsal);
    match finished_rx.recv_timeout(SOURCE_GUARD_TEST_TIMEOUT) {
        Ok(outcome) => {
            worker.join().expect("the creation worker does not panic");
            outcome
        }
        Err(failure) => {
            if let Err(payload) = worker.join() {
                std::panic::resume_unwind(payload);
            }
            panic!("the creation finishes once its installation is released: {failure:?}")
        }
    }
}

/// Retire `branch` while a creation from it is held at the guard.
fn retire_while_held(owner: &TestOwner, branch: &ProductBranchObservation) {
    let report = RuntimeWorldBranchService::retire_product_branch(owner, branch)
        .expect("the source retires while the creation is held");
    assert!(report.owner_retirement_work().is_empty());
}

/// A refused reuse installs nothing and keeps nothing: the destination
/// reservation, the operation reservation, and every obligation the held
/// reuse carried are released.
fn assert_reuse_left_nothing(owner: &TestOwner, branches: usize, obligations: usize) {
    assert_eq!(
        owner.state.branches.branch_count(),
        branches,
        "no child is installed from a head that is no longer current"
    );
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        obligations,
        "the refused child's head and observation obligations are released"
    );
}

#[test]
fn exact_reuse_whose_source_moves_before_install_is_refused_under_the_guard() {
    let (mut fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let lifecycles_before = owner_lifecycles(&owner);
    let history_before = owner.state.history.len();
    let before_hold = owner.state.retention.active_component_obligation_count();
    let held_source = source.clone();
    let mut expected_after = None;
    let denial = creation_interrupted_before_install(
        &owner,
        source.clone(),
        reuse_intent("reuse-under-moved-source"),
        |paused_owner| {
            assert_eq!(
                paused_owner.state.branches.reserved_branch_count(),
                1,
                "the reuse holds its destination reservation at the guard"
            );
            assert_eq!(
                paused_owner.state.operation.active(),
                1,
                "the reuse holds an operation reservation, so close cannot drain under it"
            );
            let reuse_cost = paused_owner
                .state
                .retention
                .active_component_obligation_count()
                - before_hold;
            assert!(reuse_cost > 0, "the held reuse carries its own obligations");
            // A different operation wins the product head this reuse was
            // admitted against, after its pre-charge recheck passed.
            seed_relational_source(paused_owner, &mut fixture, held_source);
            expected_after = Some(
                paused_owner
                    .state
                    .retention
                    .active_component_obligation_count()
                    - reuse_cost,
            );
        },
    )
    .expect_err("a source that moved before the install is refused at the install");

    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
    assert_eq!(owner_lifecycles(&owner), lifecycles_before);
    assert_eq!(
        owner.state.history.len(),
        history_before + 1,
        "the winning publication is the only history the window added"
    );
    assert_reuse_left_nothing(&owner, 1, expected_after.expect("measured while held"));
    drop(source);
    drop(fixture);
}

#[test]
fn exact_reuse_whose_source_retires_before_install_is_named_retired_under_the_guard() {
    let (_fixture, owner, root) = setup_with_relational_source(3);
    let child = create_reused_branch(&owner, &root, reuse_intent("source-retired-under-reuse"));
    let owner = Arc::new(owner);
    let lifecycles_before = owner_lifecycles(&owner);
    let before_hold = owner.state.retention.active_component_obligation_count();
    let held_child = child.clone();
    let mut expected_after = None;
    let denial = creation_interrupted_before_install(
        &owner,
        child.clone(),
        reuse_intent("reuse-from-source-retired-at-the-guard"),
        |paused_owner| {
            let reuse_cost = paused_owner
                .state
                .retention
                .active_component_obligation_count()
                - before_hold;
            retire_while_held(paused_owner, &held_child);
            expected_after = Some(
                paused_owner
                    .state
                    .retention
                    .active_component_obligation_count()
                    - reuse_cost,
            );
        },
    )
    .expect_err("a source retired before the install is refused at the install");

    assert_eq!(
        denial,
        RuntimeWorldBranchAdmissionDenial::RetiredBranch,
        "a source with no occurrence is retired, not stale, at the install too"
    );
    assert_eq!(owner_lifecycles(&owner), lifecycles_before);
    assert_reuse_left_nothing(&owner, 1, expected_after.expect("measured while held"));
    drop(child);
}

#[test]
fn a_fork_whose_source_moves_before_install_retains_the_winner_instead_of_installing() {
    let (mut fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let held_source = source.clone();
    let outcome = creation_interrupted_before_install(
        &owner,
        source.clone(),
        fork_intent(
            "branch-fork-under-moved-source",
            relational_fork("relational-branch-fork-under-moved-source"),
            signal_fork("signal-branch-fork-under-moved-source"),
        ),
        |paused_owner| {
            assert_eq!(
                paused_owner.state.custody.installed(),
                2,
                "both owner forks happened before the guard"
            );
            assert_eq!(
                paused_owner.state.branches.branch_count(),
                1,
                "the destination is not installed while held"
            );
            seed_relational_source(paused_owner, &mut fixture, held_source);
        },
    )
    .expect("a source displaced at the install is retained, not denied");

    let winner = current_root_observation(&owner);
    assert_ne!(winner.selected_commit(), source.selected_commit());
    let effects = retained_after_both_forks(outcome);
    let observed = effects
        .last_observed_head()
        .expect("the record names the head that displaced the fork");
    assert_eq!(observed.commit().identity(), winner.selected_commit());
    assert_eq!(observed.branch(), winner.branch_identity());
    assert_fork_retained_nothing_else(&owner, 1);
    assert!(owner.cleanup_recovery(effects).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
    drop(winner);
    drop(fixture);
}

#[test]
fn a_fork_whose_source_retires_before_install_is_retained_without_a_winner() {
    let (_fixture, owner, root) = setup_with_relational_source(3);
    let child = create_reused_branch(&owner, &root, reuse_intent("source-retired-under-fork"));
    let owner = Arc::new(owner);
    let held_child = child.clone();
    let outcome = creation_interrupted_before_install(
        &owner,
        child.clone(),
        fork_intent(
            "branch-fork-under-retired-source",
            relational_fork("relational-branch-fork-under-retired-source"),
            signal_fork("signal-branch-fork-under-retired-source"),
        ),
        |paused_owner| {
            assert_eq!(paused_owner.state.custody.installed(), 2);
            retire_while_held(paused_owner, &held_child);
        },
    )
    .expect("a source retired at the install is retained, not denied");

    let effects = retained_after_both_forks(outcome);
    assert!(
        effects.last_observed_head().is_none(),
        "no head displaced the fork, so the record names no winner"
    );
    assert_fork_retained_nothing_else(&owner, 1);
    assert!(owner.cleanup_recovery(effects).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
    drop(child);
}

/// A fork that performed both owner forks and was then refused at the install
/// is retained as a stale product head with both postures performed.
fn retained_after_both_forks(
    outcome: RuntimeWorldBranchCreationOutcome,
) -> ProductUnpublishedOwnerEffects {
    let effects = match outcome {
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => effects,
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a fork cannot install a child from a head that is no longer current")
        }
    };
    assert_eq!(effects.cause(), ProductUnpublishedCause::StaleProductHead);
    assert_eq!(
        effects.progress().relational_posture(),
        RelationalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.progress().signal_posture(),
        SignalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.live_obligation_count(),
        4,
        "the exact pin pair, the recovery slot, and the installed successor history"
    );
    effects
}

/// The retained fork installed no product reference, released its
/// destination and operation reservations, and kept the component branches
/// it really made in custody behind one recovery record.
fn assert_fork_retained_nothing_else(owner: &TestOwner, branches: usize) {
    assert_eq!(owner.state.branches.branch_count(), branches);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(
        owner.state.custody.installed(),
        2,
        "the component branches the fork really made stay in custody"
    );
    assert_eq!(owner.recovery_record_count(), 1);
}

#[test]
fn creation_from_a_retired_source_is_named_retired_on_every_route() {
    let (_fixture, owner, root) = setup_with_relational_source(3);
    let child = create_reused_branch(&owner, &root, reuse_intent("retired-source"));
    let report = RuntimeWorldBranchService::retire_product_branch(&owner, &child)
        .expect("the child retires");
    assert!(report.owner_retirement_work().is_empty());
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, child.branch_identity())
            .expect_err("a retired branch is not observable"),
        RuntimeWorldBranchAdmissionDenial::RetiredBranch
    );
    let lifecycles_before = owner_lifecycles(&owner);

    for intent in [
        reuse_intent("from-retired-reuse"),
        fork_intent(
            "branch-from-retired-fork",
            relational_fork("relational-branch-from-retired-fork"),
            signal_fork("signal-branch-from-retired-fork"),
        ),
    ] {
        let cancellation = RuntimeWorldCancellationSource::new();
        let denial = RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(child.clone(), intent, &cancellation.token()),
        )
        .expect_err("a retired source cannot be the source of a creation");
        assert_eq!(
            denial,
            RuntimeWorldBranchAdmissionDenial::RetiredBranch,
            "a source branch with no occurrence is retired, not stale, on every route"
        );
    }

    assert_eq!(owner_lifecycles(&owner), lifecycles_before);
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.custody.installed(), 0);
    assert_eq!(owner.recovery_record_count(), 0);
}
