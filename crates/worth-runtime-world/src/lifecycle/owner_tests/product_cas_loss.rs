//! Proofs for the losing arm of the product CAS.
//!
//! A production ready attempt is admitted and executed against an observed
//! product head. A second owner-issued publication then moves that head. The
//! ready attempt reaches `publish` against a cell that has already moved and
//! must terminate as retained owner effects naming the winner it observed.
//!
//! Both attempts execute through the real owner services. Relational and
//! Signal move independently before competing for one product reference.

use std::sync::Arc;

use crate::branch::reference_test_fixture::RealReferenceFixture;
use crate::branch::{
    ProductBranchObservation, ProductBranchReferenceCell, ProductBranchReferenceSnapshot,
};
use crate::publication::{
    CompositeLateCancellationPosture, CompositePublicationIntent, OwnerExecutionOutcome,
    RuntimeWorldCancellationSource, RuntimeWorldPublicationOutcome,
};
use crate::recovery::{
    ProductUnpublishedCause, ProductUnpublishedNextAction, ProductUnpublishedOwnerEffects,
    ProductUnpublishedRetentionPosture,
};

use super::publication::{
    prepare_relational, ready_from_prepared, setup, setup_with_relational_source, TestOwner,
};

/// One resolved race: the winner's product head, and the retained record the
/// loser produced against it.
pub(super) struct ResolvedRace {
    pub(super) owner: Arc<TestOwner>,
    pub(super) expected: ProductBranchObservation,
    pub(super) winner_head: ProductBranchReferenceSnapshot,
    pub(super) retained: ProductUnpublishedOwnerEffects,
    pub(super) history_len_after_winner: usize,
}

/// Admit and execute one ready attempt, move the product head out from under
/// it with a second owner-issued publication, then send it into `publish`.
pub(super) fn resolve_one_race(late: CompositeLateCancellationPosture) -> ResolvedRace {
    race_from(setup(), late)
}

/// The same race, run against an owner that has already completed one ordinary
/// publication. The completed attempt released its exact component custody, so
/// the registry carries a zero-count entry no live holder still names.
pub(super) fn resolve_one_race_after_a_completed_publication(
    late: CompositeLateCancellationPosture,
) -> ResolvedRace {
    race_from(setup_with_relational_source(), late)
}

fn race_from(
    base: (
        RealReferenceFixture,
        Arc<TestOwner>,
        ProductBranchObservation,
    ),
    late: CompositeLateCancellationPosture,
) -> ResolvedRace {
    let (fixture, owner, expected) = base;
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let loser = ready_from_prepared(
        owner.as_ref(),
        prepare_relational(&fixture, owner.as_ref(), expected.clone(), "cas-loser"),
        "the losing attempt settles its owner work before the race",
    );

    let winner_head = publish_competing_head(owner.as_ref(), &expected, &cell);
    assert_eq!(&winner_head, &cell.atomic_snapshot());
    assert_ne!(
        winner_head.reference_generation(),
        expected.snapshot().reference_generation(),
        "the competing publication must actually move the product reference"
    );
    let history_len_after_winner = owner.state.history.len();

    let retained = match loser.publish(&cell, late) {
        RuntimeWorldPublicationOutcome::ProductUnpublished(retained) => retained,
        other => panic!("the attempt that lost the product reference must retain: {other:?}"),
    };
    ResolvedRace {
        owner,
        expected,
        winner_head,
        retained,
        history_len_after_winner,
    }
}

/// Execute the independent Signal-only competitor through the same production
/// preparation, execution, retention and CAS path as the Relational loser.
fn publish_competing_head(
    owner: &TestOwner,
    current: &ProductBranchObservation,
    cell: &ProductBranchReferenceCell,
) -> ProductBranchReferenceSnapshot {
    use crate::lifecycle::{RuntimeWorldOwnerExecutionService, RuntimeWorldPreparationService};
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .prepare_publication(
            current.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .expect("the exact product head admits the independent Signal attempt");
    let settlement =
        match owner.execute_with_signal(prepared, &mut (), &cancellation.token(), |_| Ok(())) {
            OwnerExecutionOutcome::Settled(settlement) => settlement,
            other => panic!("the Signal competitor must settle its real effect: {other:?}"),
        };
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement
        .ready(successor)
        .expect("the exact Signal successor is retained");
    match ready.publish(cell, CompositeLateCancellationPosture::NotRequested) {
        RuntimeWorldPublicationOutcome::Performed(performed) => {
            performed.new_product_head().clone()
        }
        other => panic!("the Signal competitor must win product publication: {other:?}"),
    }
}

/// SPEC-P4-016. The expected-observation comparison precedes materialization,
/// so the attempt that lost the product reference never takes product-head
/// authority: it advances no product reference generation, leaves the winner's
/// snapshot as the sole product head, and leaks no reserved capacity.
///
/// Its reserved history slot is released rather than spent; that is proved on
/// its own by `a_losing_cas_attempt_does_not_consume_its_reserved_history_slot`.
#[test]
fn a_losing_cas_attempt_takes_no_product_head_authority_and_advances_no_generation() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);
    let cell = race
        .owner
        .state
        .branches
        .root_cell()
        .expect("bootstrapped cell");

    assert_eq!(
        cell.atomic_snapshot(),
        race.winner_head,
        "the loser must not move the product reference"
    );
    assert_eq!(
        cell.atomic_snapshot().reference_generation(),
        race.winner_head.reference_generation(),
        "the loser must not consume a product reference generation"
    );
    assert_eq!(
        cell.atomic_snapshot().selected_commit(),
        race.winner_head.selected_commit(),
        "the product head reaches the winner's commit alone"
    );
    assert_eq!(
        race.owner.state.recovery.reserved_slots(),
        0,
        "the loser leaks no reserved recovery capacity"
    );
    assert_eq!(race.owner.state.operation.active(), 0);
    assert_eq!(race.owner.state.publication_capacity.active(), 0);

    let handle = race.retained.recovery_handle();
    drop(race.retained);
    assert!(race.owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(race.owner.recovery_record_count(), 0);
    assert_eq!(
        cell.atomic_snapshot(),
        race.winner_head,
        "cleaning up the loser leaves the winner's product head untouched"
    );
}

/// The retained record names the settled owner occurrence it produced and the
/// exact winner it observed, and derives its next actions from that progress.
#[test]
fn lost_product_cas_retains_the_settled_owner_occurrence_and_observed_winner() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);

    assert_eq!(
        race.retained.cause(),
        ProductUnpublishedCause::ProductPublicationLost
    );
    assert_eq!(
        race.retained.last_observed_head(),
        Some(&race.winner_head),
        "the retained record names the exact winner it observed"
    );
    assert_eq!(
        race.retained.expected_head().snapshot(),
        race.expected.snapshot(),
        "the retained record keeps the head the attempt admitted against"
    );
    assert_eq!(
        race.retained.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::Settled,
        "the loser's own owner occurrence is settled, not rolled back"
    );
    assert_eq!(
        race.retained.next_actions(),
        crate::recovery::next_actions_for_progress(race.retained.progress(), race.retained.cause()),
        "next actions are derived from the attempt's progress, never hard-coded"
    );
    assert!(!race
        .retained
        .next_actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
}

/// A caller that reports cancellation in the same window as the loss must not
/// have its loss reclassified as cancellation.
#[test]
fn cancelled_ready_loses_to_winner_and_retains_publication_loss() {
    let race = resolve_one_race(CompositeLateCancellationPosture::RequestedBeforeProductMovement);

    assert_eq!(
        race.retained.cause(),
        ProductUnpublishedCause::ProductPublicationLost,
        "an observed loss is never hidden as cancellation"
    );
    assert_eq!(race.retained.last_observed_head(), Some(&race.winner_head));
    assert_eq!(
        race.owner
            .state
            .branches
            .root_cell()
            .expect("bootstrapped cell")
            .atomic_snapshot(),
        race.winner_head
    );
}

/// The loser keeps its exact commit results and its exact component custody:
/// the successor occurrence stays reachable and the pin pair stays held.
#[test]
fn complete_ready_publish_path_retains_commit_results_and_custody_on_observed_product_loss() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);

    assert_eq!(
        race.retained.retention_posture(),
        ProductUnpublishedRetentionPosture::RetainedExact,
        "a lost CAS keeps the exact component pins it already held"
    );
    assert_eq!(
        race.retained.live_obligation_count(),
        3,
        "a record that installed no successor keeps its two component pins and the recovery slot alone"
    );
    assert_eq!(race.retained.owner_effect_count(), 1);
    assert_eq!(
        race.retained.successor_commit(),
        None,
        "a pre-movement loser names no installed successor occurrence"
    );
    assert_ne!(
        race.retained
            .successor_basis()
            .expect("a lost CAS retains its successor basis"),
        race.expected.basis(),
        "the successor this attempt would have published is retained as basis          evidence: the owner occurrence it settled, not the head it admitted          against"
    );
    assert_eq!(race.owner.recovery_record_count(), 1);
}

/// The reserved history slot of an attempt that lost before the product
/// reference moved goes back to the catalog. Installing it would charge a
/// bounded slot for a commit no product head names and no caller can reach,
/// and the retained record needs no installed occurrence to stay honest: the
/// successor basis is the evidence a fresh attempt names it by.
#[test]
fn a_losing_cas_attempt_does_not_consume_its_reserved_history_slot() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);

    assert_eq!(
        race.owner.state.history.len(),
        race.history_len_after_winner,
        "the loser installs nothing into history"
    );
    assert_eq!(
        race.owner.state.history.reserved_len(),
        0,
        "the loser's history reservation is released, not held"
    );
    assert_eq!(
        race.retained.successor_commit(),
        None,
        "no installed successor occurrence is retained"
    );

    let handle = race.retained.recovery_handle();
    drop(race.retained);
    assert!(race.owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(
        race.owner.state.history.len(),
        race.history_len_after_winner,
        "cleanup has no successor occurrence to remove and removes none"
    );
    assert_eq!(race.owner.state.history.reserved_len(), 0);
}
