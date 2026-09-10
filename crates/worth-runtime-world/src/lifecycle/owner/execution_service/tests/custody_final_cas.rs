use super::*;
use crate::publication::AttemptProductMovementFailure;
use crate::recovery::ProductUnpublishedRetentionPosture;

#[test]
fn final_cell_comparison_rejects_a_real_winner_without_promoting_or_retagging() {
    let (fixture, owner, expected) = setup();
    let winner = ready_relational_competitor(&fixture, &owner, &expected, "final-cell-winner");
    let settlement = settled(execute_without_signal(
        &owner,
        prepare_relational(&fixture, &owner, &expected, "final-cell-loser"),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    let (_, results) = settlement.progress().ready_results().unwrap();
    // Borrow the real attempt's custody at the final-CAS boundary, beyond the
    // public preflight comparison. This deterministically puts a real product
    // winner between admission and the final branch write-lock comparison.
    let (attempt, _) = settlement.into_parts();
    let parts = attempt.into_parts();
    let mut custody = parts.custody;
    let mut counters = parts.counters;
    let commit = custody.prepare_commit(successor, &results);
    custody.bind_publication_pins(commit.basis()).unwrap();
    let selected = publish_ready_competing_head(&owner, winner, &expected);
    let cell = owner.state.branches.root_cell().unwrap();
    let before_history = owner.state.history.counters();
    let before_counters = counters;
    let loss = custody
        .attempt_movement(
            &expected,
            &commit,
            &results,
            &mut counters,
            CompositeLateCancellationPosture::NotRequested,
            &cell,
            None,
        )
        .unwrap_err();
    let AttemptProductMovementFailure::Reference(loss) = loss else {
        panic!("a final cell competitor must produce a reference loss");
    };
    assert_eq!(loss.observed_head(), selected.snapshot());
    assert_eq!(
        owner.state.history.counters().metadata_promotions(),
        before_history.metadata_promotions()
    );
    assert_eq!(
        counters.history_slots_installed(),
        before_counters.history_slots_installed()
    );
    assert!(owner.state.history.lookup(commit.identity()).is_none());
    assert_eq!(owner.state.history.reserved_len(), 1);
    drop(custody);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::PublicationPinsRetained,
        "the losing final comparison never minted product-head claims"
    );
    assert!(record.successor_commit().is_none());
    assert_eq!(cell.atomic_snapshot(), *selected.snapshot());
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
}
