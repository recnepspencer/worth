//! Unit tests exercising `PinTable` in-flight custody against concurrent pruning.
use super::PinTable;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity,
};

fn commit(ordinal: u64) -> DurableCompositeCommitIdentity {
    let home = HomeIdentity::from_observed_bytes([7; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    DurableCompositeCommitIdentity::from_recorded(owner, ordinal)
}

#[test]
fn in_flight_pin_table_survives_concurrent_prune_until_released_and_tracks_activity() {
    let table = PinTable::<()>::default();
    let c1 = commit(1);
    let c2 = commit(2);

    // Request A acquires in-flight pin for c1 before its journal frame exists.
    table.insert_and_increment_in_flight(c1, ());
    // Durable pin c2 has no in-flight holder.
    table.pin_durable(c2, ());

    // Concurrent Request B prunes while neither c1 nor c2 is active in the ledger.
    table.prune(|_| false);
    assert!(
        table.has_pin(c1),
        "in-flight holder must prevent concurrent prune before journal staging"
    );
    assert!(
        !table.has_pin(c2),
        "discharged pin with zero in-flight holders must be pruned"
    );

    // A second concurrent in-flight caller shares c1 via try_increment_in_flight.
    assert!(table.try_increment_in_flight(c1));
    assert!(!table.try_increment_in_flight(c2));

    // Releasing the first holder still leaves in_flight == 1, so c1 stays even if inactive.
    table.release_in_flight(c1, |_| false);
    assert!(table.has_pin(c1));

    // Releasing the second holder when c1 IS active in the journal preserves c1.
    table.release_in_flight(c1, |id| id == c1);
    assert!(table.has_pin(c1));

    // Subsequent acknowledged discharge prunes c1 once still_active becomes false.
    table.prune(|_| false);
    assert!(!table.has_pin(c1));
}

#[test]
fn fixed_source_supersession_and_branch_retirement_pin_lifecycle() {
    use crate::domain_computation::primary_graph::query_obligations::{
        history::QueryObligationJournalState, QueryObligationLedger,
        QueryObligationPublicationGenerations, QueryOutputSourceObligation,
        QueryOutputSourceRecord, QueryProgramSupportRecord,
    };
    use worth_foundational::facade::CanonicalDigestId;

    let home = HomeIdentity::from_observed_bytes([9; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    let c1 = DurableCompositeCommitIdentity::from_recorded(owner, 1);
    let c2 = DurableCompositeCommitIdentity::from_recorded(owner, 2);
    let revision = CanonicalDigestId::new([1; 32]);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let install = ledger
        .prepare(
            QueryProgramSupportRecord::Install { revision },
            &mut generations,
        )
        .unwrap();
    ledger.readmit(install).unwrap();
    let make_row = |commit| QueryOutputSourceObligation {
        commit,
        revision,
        root: "root".to_owned(),
        branch: "main".to_owned(),
        lifecycle: 1,
        generation: commit.ordinal(),
        scope: (0, 1, 1),
        key: [2; 32],
        intent: "intent".to_owned(),
        bindings: None,
        discovery_recipe: None,
    };
    let f1 = ledger
        .prepare(
            QueryOutputSourceRecord::Retain(make_row(c1)),
            &mut generations,
        )
        .unwrap();
    ledger.readmit(f1).unwrap();
    let f2 = ledger
        .prepare(
            QueryOutputSourceRecord::Retain(make_row(c2)),
            &mut generations,
        )
        .unwrap();
    ledger.readmit(f2).unwrap();
    let journal = QueryObligationJournalState::from_ledger(ledger);
    let output = worth_query_execution_obligations::facade::ledger_inspect::output(&journal.ledger);
    assert!(
        !output.sources.contains_key(&c1) && output.sources.contains_key(&c2),
        "newer fixed source must supersede prior unbound scope in ledger"
    );

    // PinTable pruned against post-supersession ledger activity releases c1 and keeps c2.
    let table = PinTable::<()>::default();
    table.pin_durable(c1, ());
    table.pin_durable(c2, ());
    table.prune(|id| output.sources.contains_key(&id));
    assert!(!table.has_pin(c1));
    assert!(table.has_pin(c2));
}
