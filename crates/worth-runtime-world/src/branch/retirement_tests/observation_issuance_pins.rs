//! The observation a fork finalization issues joins the exact component slots
//! the fork's own publication already pinned on the destination basis.
//!
//! The issuance happens after the product movement with no token reserved for
//! it, and it is safe only because it can add no unique pin. This proof
//! isolates the issuance and the product-head transfer that follows it, in
//! one world, by taking the same fork twice: once with the observation
//! authority withheld by the rehearsal seam, which stops before both, and
//! once with it issued. Each route must add exactly the pin pair the two
//! forked owners' destination basis costs, so neither step adds anything.

use std::sync::{mpsc, Arc};
use std::time::Duration;

use crate::lifecycle::{
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest, RuntimeWorldBranchService,
};
use crate::publication::RuntimeWorldCancellationSource;
use crate::recovery::ProductUnpublishedOwnerEffects;

use super::fork_creation::{
    create_forked_branch, fork_intent, relational_fork, setup_with_relational_source, signal_fork,
};

const OBSERVATION_PIN_TEST_TIMEOUT: Duration = Duration::from_secs(2);

/// One slot per forked owner: the destination basis differs from the source
/// on both component axes.
const FORKED_OWNER_SLOTS: usize = 2;

#[test]
fn observation_issuance_adds_no_unique_pin_beyond_the_withheld_route() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let pins_before = owner.state.retention.unique_pin_count();

    let (pins_at_withheld_boundary, effects) =
        fork_with_observation_withheld(&owner, &source, "observation-pins-withheld");
    let pins_after_withheld = owner.state.retention.unique_pin_count();

    let child = create_forked_branch(
        owner.as_ref(),
        &source,
        fork_intent(
            "branch-observation-pins-issued",
            relational_fork("relational-branch-observation-pins-issued"),
            signal_fork("signal-branch-observation-pins-issued"),
        ),
    );
    let pins_after_issued = owner.state.retention.unique_pin_count();

    assert_eq!(
        pins_at_withheld_boundary,
        pins_before + FORKED_OWNER_SLOTS,
        "the product movement alone pins the destination pair"
    );
    assert_eq!(
        pins_after_withheld,
        pins_before + FORKED_OWNER_SLOTS,
        "the retained record keeps exactly that pair"
    );
    assert_eq!(
        pins_after_issued,
        pins_after_withheld + FORKED_OWNER_SLOTS,
        "issuing the observation on top of the same movement adds no unique pin"
    );
    drop(child);
    assert!(owner.cleanup_recovery(effects).is_some());
    drop(fixture);
}

/// Run one both-owner fork whose observation authority is withheld, read the
/// unique pin count while the attempt is held after its product movement and
/// before its recovery record exists, then let it retain.
fn fork_with_observation_withheld(
    owner: &Arc<super::TestOwner>,
    source: &crate::branch::ProductBranchObservation,
    name: &str,
) -> (usize, ProductUnpublishedOwnerEffects) {
    let (reached_tx, reached_rx) = mpsc::sync_channel(1);
    let rehearsal = owner.rehearse_forked_finalization_recovery(reached_tx);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    let worker_owner = Arc::clone(owner);
    let source = source.clone();
    let intent = fork_intent(
        &format!("branch-{name}"),
        relational_fork(&format!("relational-branch-{name}")),
        signal_fork(&format!("signal-branch-{name}")),
    );
    let worker = std::thread::spawn(move || {
        let cancellation = RuntimeWorldCancellationSource::new();
        let outcome = RuntimeWorldBranchService::create_product_branch(
            worker_owner.as_ref(),
            RuntimeWorldBranchCreationRequest::new(source, intent, &cancellation.token()),
        );
        finished_tx
            .send(outcome)
            .expect("the proof still owns its completion receiver");
    });
    reached_rx
        .recv_timeout(OBSERVATION_PIN_TEST_TIMEOUT)
        .expect("the withheld fork reaches its recovery-record boundary");
    let pins_at_boundary = owner.state.retention.unique_pin_count();
    drop(rehearsal);
    let outcome = finished_rx
        .recv_timeout(OBSERVATION_PIN_TEST_TIMEOUT)
        .expect("the withheld fork finishes once released")
        .expect("a withheld observation authority is a product-unpublished outcome");
    worker
        .join()
        .expect("the withheld fork worker does not panic");
    match outcome {
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
            (pins_at_boundary, effects)
        }
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a fork without observation authority cannot publish its destination")
        }
    }
}
