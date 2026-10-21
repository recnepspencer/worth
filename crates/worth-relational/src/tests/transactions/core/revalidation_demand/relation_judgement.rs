//! Relation-only judgment must read the unchanged native preimage.
use super::{fixtures::STRICT_MODE_NAME, relation_fixtures::*};
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as Policy;

#[test]
fn relation_demand_judges_exact_nonzero_slot_under_proposed_rules_without_rewriting() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime();
    let source = create_entity_in_partition(execution, &runtime, "lax", PartitionId(1));
    let target = create_entity_in_partition(execution, &runtime, "target", PartitionId(2));
    let second_target =
        create_entity_in_partition(execution, &runtime, "second-target", PartitionId(2));
    let bad = create_relation(execution, &runtime, source, target, "oversized");
    let good = create_relation(execution, &runtime, source, second_target, "ok");
    assert_ne!(good.slot_index(), 0);
    // An entity-only edit does not select this relation-only rule. The read
    // demand below is therefore necessary, rather than redundant validation.
    update_entity_and_release_snapshot(execution, &runtime, source, STRICT_MODE_NAME);
    let before = runtime
        .read_truth()
        .read_version(execution, runtime.current_version_id())
        .unwrap();
    let before_record = before.get_relation(good).unwrap().clone();
    let before_history = relation_aspect_history_digest(&runtime, good, None);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch(good), Policy::SystemAllocation)
        .unwrap();
    assert_eq!(transaction.footprint().writes().len(), 0);
    assert!(transaction.footprint().reads().any(|read| matches!(read,
        crate::mvcc::RelationalTransactionReadLocus::Existing(crate::transactions::data::RecordRef::Relation(id)) if *id == good)));
    let planned = transaction
        .plan_bulk_mutation_batch(&runtime)
        .unwrap()
        .unwrap();
    assert_eq!(planned.locality.relation_target_count, 1);
    assert_eq!(planned.locality.entity_target_count, 0);
    let inspected = transaction.inspect_staging();
    assert_eq!(inspected.intent_counts.relation_revalidation_count, 1);
    assert_eq!(inspected.intent_counts.relation_mutation_count, 0);
    let outcome = transaction.commit(execution, &runtime).unwrap();
    assert!(outcome.changed_records.is_empty());
    assert!(outcome.patch().is_empty());
    assert_eq!(
        outcome.structural_summary().invariant_groups,
        crate::validation::data::InvariantGroupSet::empty()
    );
    release_test_commit_snapshot(&runtime, &outcome);
    let after = runtime
        .read_truth()
        .read_version(execution, runtime.current_version_id())
        .unwrap();
    assert_eq!(
        after.get_relation(good),
        Some(&before_record),
        "payload/endpoints/lifecycle/version/lineage unchanged"
    );
    let after_history = relation_aspect_history_digest(&runtime, good, None);
    assert_eq!(after_history.entry_count, before_history.entry_count);
    assert_eq!(
        after_history.opaque_aspect_entry_count,
        before_history.opaque_aspect_entry_count
    );
    assert_eq!(
        after_history.resolved_aspects,
        before_history.resolved_aspects
    );
    assert_eq!(
        after_history.traversed_commits,
        before_history.traversed_commits + 1,
        "judgment publishes a commit, without adding a record-history entry"
    );
    let before_refusal = runtime.current_version_id();
    let mut rejected = test_owner_begin_transaction_for_main(&runtime);
    rejected
        .push_batch(batch(bad), Policy::SystemAllocation)
        .unwrap();
    assert_rule(rejected.commit(execution, &runtime).unwrap_err());
    assert_eq!(runtime.current_version_id(), before_refusal);
    assert_eq!(
        after.get_relation(good),
        before.get_relation(good),
        "held original image unchanged"
    );
}

#[test]
fn relation_demand_requires_the_actual_live_record() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime();
    let source = create_entity(execution, &runtime, "lax");
    let present = create_relation(execution, &runtime, source, source, "ok");
    let absent = RelationId::new(
        present.partition_id,
        present.local_slot_value() + 4096,
        present.generation_value(),
    );
    let before = runtime.current_version_id();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch(absent), Policy::SystemAllocation)
        .unwrap();
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(execution, &runtime).unwrap_err()
    else {
        panic!("expected stale-target refusal")
    };
    assert!(
        matches!(error.class, crate::transactions::data::ConflictClass::StaleTarget {
        target: crate::transactions::data::ExistingRecordTarget::Relation(id), ..
    } if id == absent)
    );
    assert_eq!(runtime.current_version_id(), before);
}
