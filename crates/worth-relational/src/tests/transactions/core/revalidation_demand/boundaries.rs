//! A revalidation demand is bounded and accounted like every other demand a
//! transaction stages.

use super::fixtures::*;
use crate::facade::mvcc::RelationalTransactionStagingDenial;
use crate::tests::support::*;

#[test]
fn staged_demands_consume_footprint_capacity_and_exhaust_it_by_name() {
    let runtime = strictness_runtime_with_footprint_ceiling(4);
    let records = [
        create_entity(&runtime, COMPLIANT_NAME),
        create_entity(&runtime, "second"),
        create_entity(&runtime, "third"),
        create_entity(&runtime, "fourth"),
        create_entity(&runtime, "fifth"),
    ];

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("within-ceiling", records[..4].to_vec()))
        .expect("four demands fit the ceiling exactly");

    assert_eq!(
        transaction.push_batch(revalidation_batch("over-ceiling", [records[4]])),
        Err(
            RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                maximum_loci: 4,
                required_loci: 5,
            }
        ),
        "a fifth demand must be refused by name, not admitted unbounded"
    );
}

#[test]
fn a_refused_demand_leaves_no_staging_residue() {
    let runtime = strictness_runtime_with_footprint_ceiling(4);
    let first = create_entity(&runtime, COMPLIANT_NAME);
    let second = create_entity(&runtime, "second");
    let third = create_entity(&runtime, "third");
    let fourth = create_entity(&runtime, "fourth");
    let fifth = create_entity(&runtime, "fifth");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch(
            "within-ceiling",
            [first, second, third, fourth],
        ))
        .expect("four demands fit the ceiling");
    let batch_count = transaction.batches().len();
    let read_count = transaction.footprint().reads().len();

    assert!(transaction
        .push_batch(revalidation_batch("over-ceiling", [fifth]))
        .is_err());
    assert_eq!(transaction.batches().len(), batch_count);
    assert_eq!(transaction.footprint().reads().len(), read_count);
}

#[test]
fn a_demand_claims_a_read_locus_and_never_a_write_locus() {
    let runtime = strictness_runtime();
    let entity = create_entity(&runtime, COMPLIANT_NAME);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("observe-only", [entity]))
        .expect("staging stays within configured resource budgets");

    assert_eq!(
        transaction.footprint().writes().len(),
        0,
        "a demand writes nothing, so it must claim no write locus"
    );
    assert!(
        transaction.footprint().reads().any(|locus| matches!(
            locus,
            crate::facade::mvcc::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity_id),
            ) if *entity_id == entity
        )),
        "a demand observes the record, so it must claim a read locus: {:?}",
        transaction.footprint().reads().collect::<Vec<_>>()
    );
}

#[test]
fn rolling_back_a_savepoint_past_a_demand_reports_no_restoration() {
    let runtime = strictness_runtime();
    let entity = create_entity(&runtime, COMPLIANT_NAME);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    let savepoint = transaction
        .create_savepoint()
        .expect("savepoint within configured budget");
    transaction
        .push_batch(revalidation_batch("discarded", [entity]))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .rollback_to_savepoint(savepoint)
        .expect("rollback to the savepoint");

    assert!(
        !outcome.has_effects(),
        "discarding a demand restores nothing, so it reports no effect: {:?}",
        outcome.effects()
    );
}

#[test]
fn a_demand_puts_exactly_its_record_in_front_of_the_platform_touched_slot_rules() {
    let runtime = strictness_runtime();
    for index in 0..4 {
        let _ = create_entity(&runtime, &format!("bystander-{index}"));
    }
    let target = create_entity(&runtime, COMPLIANT_NAME);
    assert_ne!(
        target.slot_index(),
        0,
        "the flagship touched-slot court must not degenerate to the arena's first slot"
    );

    runtime.performance_access().reset_counters();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(revalidation_batch("revalidate-target", [target]))
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(&runtime)
        .expect("the compliant record passes the strict reading");
    let counters = runtime.performance_access().counters();
    release_test_commit_snapshot(&runtime, &outcome);

    assert_eq!(
        counters.entity_slots_touched_by_commit, 1,
        "the counter proves one slot entered the journal route; slot_identity owns which slot"
    );
    assert_eq!(
        counters.invariant_entity_slot_scans, 1,
        "the counter proves one journalled slot was scanned; slot_identity owns its identity"
    );
}
