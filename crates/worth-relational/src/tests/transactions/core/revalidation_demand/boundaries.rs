//! Revalidation demands preserve observation, rollback, and touched-slot semantics.

use super::fixtures::*;
use crate::tests::support::*;

#[test]
fn a_refused_demand_leaves_no_staging_residue() {
    use worth_execution::{CancellationToken, ExecutionAllocationPolicy as Policy, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let runtime = strictness_runtime();
    let entity = create_entity(&runtime, COMPLIANT_NAME);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    let before_batches = transaction.batches().to_vec();
    let before_reads = transaction.footprint().clone();
    let lease = test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), 0, 1),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let failure = transaction
        .push_batch(
            revalidation_batch("must-not-stage", [entity]),
            Policy::Execution(&lease),
        )
        .unwrap_err();
    assert!(
        failure.allocation_denial().is_some(),
        "the actual demand door refuses native backing"
    );
    assert_eq!(transaction.batches(), before_batches);
    assert_eq!(transaction.footprint(), &before_reads);
    transaction
        .push_batch(
            revalidation_batch("retry", [entity]),
            Policy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(transaction.batches().len(), before_batches.len() + 1);
}

#[test]
fn a_demand_claims_a_read_locus_and_never_a_write_locus() {
    let runtime = strictness_runtime();
    let entity = create_entity(&runtime, COMPLIANT_NAME);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            revalidation_batch("observe-only", [entity]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
        .push_batch(
            revalidation_batch("discarded", [entity]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
        .push_batch(
            revalidation_batch("revalidate-target", [target]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("staging stays within configured resource budgets");
    let outcome = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
