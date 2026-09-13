//! Close admission across a forked-branch product-unpublished finalization.
//!
//! A fork that settles its owner work and then loses an owner-issued authority
//! keeps real component custody, so it must enter recovery and stay there. The
//! only custody covering the gap between `begin_recovery()` and the installed
//! recovery record is the operation reservation itself: while it lives the
//! ledger reports one recovering operation, and `close()` must answer
//! `InFlightCriticalSection`; once the record exists the installed slot takes
//! over.
//!
//! The route is reached by withholding the observation authority that
//! `issue_observation_authority` already models as a denial. Every other
//! finalization denial (publication binding, commit capacity, recovery slot) is
//! worst-case reserved before the attempt runs, so no honest budget can starve
//! one deterministically; the rehearsal seam is keyed to this owner and armed
//! for exactly one attempt.

use std::sync::{mpsc, Arc};
use std::time::Duration;

use crate::lifecycle::{
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest,
    RuntimeWorldBranchService, RuntimeWorldCloseDenial, RuntimeWorldOwnerLifecycleObservation,
};
use crate::publication::{
    RelationalAttemptProgressPosture, RuntimeWorldCancellationSource, SignalAttemptProgressPosture,
};
use crate::recovery::{
    next_actions_for_progress, ProductUnpublishedCause, ProductUnpublishedNextAction,
    ProductUnpublishedOwnerEffects,
};

use super::fork_creation::{
    fork_intent, relational_fork, setup_with_relational_source, signal_fork,
};

const FORK_FINALIZATION_TEST_TIMEOUT: Duration = Duration::from_secs(2);

#[test]
fn forked_finalization_recovery_denies_close_until_its_record_is_installed() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let branches_before = owner.state.branches.branch_count();
    let intent = fork_intent(
        "branch-fork-finalization-race",
        relational_fork("relational-branch-fork-finalization-race"),
        signal_fork("signal-branch-fork-finalization-race"),
    );

    let (reached_tx, reached_rx) = mpsc::sync_channel(1);
    let rehearsal = owner.rehearse_forked_finalization_recovery(reached_tx);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    let worker_owner = Arc::clone(&owner);
    let worker = std::thread::spawn(move || {
        let cancellation = RuntimeWorldCancellationSource::new();
        let outcome = RuntimeWorldBranchService::create_product_branch(
            worker_owner.as_ref(),
            RuntimeWorldBranchCreationRequest::new(source, intent, &cancellation.token()),
        );
        finished_tx
            .send(outcome)
            .expect("the finalization proof still owns its completion receiver");
    });

    let paused = reached_rx
        .recv_timeout(FORK_FINALIZATION_TEST_TIMEOUT)
        .expect("the fork reaches its recovery-record construction boundary");
    assert_eq!(paused.owner_identity(), owner.owner_identity());
    assert_close_denied_before_any_record(&owner);

    drop(rehearsal);
    let outcome = finished_rx
        .recv_timeout(FORK_FINALIZATION_TEST_TIMEOUT)
        .expect("the fork finishes after its recovery boundary is released")
        .expect("a withheld observation authority is a product-unpublished outcome");
    worker.join().expect("the fork worker does not panic");
    let effects = match outcome {
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => effects,
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a fork without observation authority cannot publish its destination")
        }
    };

    assert_eq!(
        effects.cause(),
        ProductUnpublishedCause::DestinationAdmissionDenied
    );
    assert_finalization_record_contract(&effects);
    assert_retained_custody_survives_the_reservation(&owner, branches_before);
    assert_cleanup_reopens_close(&owner, effects);
    drop(fixture);
}

/// While the diverted attempt is held before its record exists, the recovering
/// operation reservation is the only custody denying `close()`.
fn assert_close_denied_before_any_record(owner: &super::TestOwner) {
    assert_eq!(
        owner.recovery_record_count(),
        0,
        "the paused attempt has not installed its record yet"
    );
    assert_eq!(
        owner
            .close()
            .expect_err("the recovering reservation alone must deny close"),
        RuntimeWorldCloseDenial::InFlightCriticalSection,
        "the recovering reservation alone must deny close before the record exists"
    );
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open,
        "a denied close must not enter Closing"
    );
    assert_eq!(owner.state.operation.active(), 1);
    assert_eq!(recovery_active(owner), 1);
}

/// Once the record is installed the reservation is gone, so the close denial
/// ends with it, and the destination is left uninstalled.
fn assert_retained_custody_survives_the_reservation(
    owner: &super::TestOwner,
    branches_before: usize,
) {
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(recovery_active(owner), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    // SPEC-P4-008: the reservation's denial ends with the reservation. The
    // installed record it left behind is exposed by close rather than refused,
    // and that exposure is pinned by
    // `close_exposes_every_retained_record_in_its_terminal_report`, so this
    // proof keeps its world open for the cleanup step below.
    assert_eq!(
        owner.state.branches.reserved_branch_count(),
        0,
        "the destination reservation is released with the retained attempt"
    );
    assert_eq!(
        owner.state.branches.branch_count(),
        branches_before,
        "no product branch is installed for a retained fork"
    );
}

/// Clearing the retained record is the last close obligation the diverted fork
/// created.
fn assert_cleanup_reopens_close(owner: &super::TestOwner, effects: ProductUnpublishedOwnerEffects) {
    assert!(
        owner.cleanup_recovery(effects).is_some(),
        "the retained finalization record is released by its own capability"
    );
    assert_eq!(owner.recovery_record_count(), 0);
    let _report = owner
        .close()
        .expect("cleanup removes the final recovery close obligation");
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
}

/// A finalization-path record must carry the same retained accounting and the
/// same derived continuation as the publication retention route.
///
/// `SettleOwnerEffects` cannot appear here: `into_ready_results` admits a fork
/// to finalization only when Relational is fork-only or settled, and it then
/// hands finalization a progress summary whose evidence is already consumed, so
/// `relational_requires_settlement()` is structurally false on this route. The
/// assertion therefore pins the derived slice, not a literal, against the same
/// authority the publication route uses.
///
/// Destination admission failure does not establish owner loss, so it must
/// not advertise owner closure as a recovery action.
fn assert_finalization_record_contract(effects: &ProductUnpublishedOwnerEffects) {
    assert_eq!(
        effects.progress().relational_posture(),
        RelationalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.progress().signal_posture(),
        SignalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.next_actions(),
        next_actions_for_progress(effects.progress(), effects.cause()).as_slice(),
        "finalization derives its continuation instead of restating a literal"
    );
    assert_eq!(
        effects.cause(),
        ProductUnpublishedCause::DestinationAdmissionDenied
    );
    assert_eq!(
        effects.next_actions(),
        [
            ProductUnpublishedNextAction::ReleaseObligations,
            ProductUnpublishedNextAction::Inspect,
        ]
    );
    assert_eq!(
        effects.live_obligation_count(),
        4,
        "the exact pin pair, the recovery slot, and the installed successor history"
    );
    assert_eq!(
        effects.metadata_bytes(),
        ProductUnpublishedOwnerEffects::metadata_charge_hint()
    );
}

fn recovery_active(owner: &super::TestOwner) -> usize {
    owner
        .state
        .operation
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .recovery_active
}
