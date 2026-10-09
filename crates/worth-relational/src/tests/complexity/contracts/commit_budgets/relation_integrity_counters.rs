use super::*;

#[test]
fn complexity_budget_relation_integrity_skips_entity_only_mutation_work() {
    let runtime = relation_integrity_cardinality_runtime();
    let entity = create_entity(&runtime, "entity-only");

    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, entity, "entity-only-updated");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.relation_integrity_contracts_evaluated, 0);
    assert_eq!(counters.relation_endpoint_kind_checks, 0);
    assert_eq!(counters.relation_cardinality_checks, 0);
    assert_eq!(counters.relation_uniqueness_checks, 0);
    assert_eq!(counters.relation_symmetry_checks, 0);
    assert_eq!(counters.relation_endpoint_deletion_checks, 0);
}

#[test]
fn complexity_budget_relation_integrity_uniqueness_uses_adjacency_local_candidates() {
    let runtime = relation_integrity_uniqueness_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let _existing = create_relation(&runtime, source, target, "existing");
    for index in 0..10 {
        let other_source = create_entity(&runtime, &format!("other-source-{index}"));
        let other_target = create_entity(&runtime, &format!("other-target-{index}"));
        let _ = create_relation(
            &runtime,
            other_source,
            other_target,
            &format!("other-rel-{index}"),
        );
    }

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-unique-relation").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("duplicate"),
                source: crate::transactions::data::EntityReference::Existing(target),
                target: crate::transactions::data::EntityReference::Existing(source),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::RelationUniquenessViolation
    ));
    assert_eq!(counters.relation_integrity_contracts_evaluated, 1);
    assert_eq!(counters.relation_uniqueness_checks, 1);
    assert_eq!(counters.relation_uniqueness_candidates_scanned, 1);
}

#[test]
fn complexity_budget_relation_integrity_symmetry_checks_only_touched_pairs() {
    let runtime = relation_integrity_symmetry_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("missing-twin").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("missing-twin"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::RelationSymmetryViolation
    ));
    assert_eq!(counters.relation_integrity_contracts_evaluated, 1);
    assert_eq!(counters.relation_symmetry_checks, 1);
    assert_eq!(counters.relation_uniqueness_candidates_scanned, 0);
}

#[test]
fn complexity_budget_relation_integrity_endpoint_deletion_checks_only_deleted_endpoints() {
    let runtime = relation_integrity_endpoint_deletion_runtime();
    let (source, _target, _relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::RelationEndpointDeletionIntegrityViolation
    ));
    assert_eq!(counters.relation_integrity_contracts_evaluated, 1);
    assert_eq!(counters.relation_endpoint_deletion_checks, 1);
    assert_eq!(counters.relation_symmetry_checks, 0);
}

#[test]
fn complexity_budget_relation_integrity_reuses_touched_scope_across_multiple_contracts() {
    let runtime = relation_integrity_multi_contract_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let _existing = create_relation(&runtime, source, target, "existing");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-and-missing-twin").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("duplicate"),
                source: crate::transactions::data::EntityReference::Existing(target),
                target: crate::transactions::data::EntityReference::Existing(source),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let _error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.relation_integrity_contracts_evaluated, 3);
    assert_eq!(
        counters.relation_uniqueness_candidates_scanned,
        1,
        "touched live relation scope should be scanned once per relation kind, not once per contract"
    );
}

#[test]
fn complexity_budget_relation_integrity_minimum_certification_reports_snapshot_breadth() {
    let runtime = relation_integrity_minimum_certification_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    create_relation(&runtime, source, target, "single");

    runtime.performance_access().reset_counters();
    let result = runtime.validation().certification_state();
    let counters = runtime.performance_access().counters();

    assert!(result.summary().publication_failure().is_some());
    assert_eq!(
        counters.relation_cardinality_minimum_certification_contracts_evaluated,
        1
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_relation_slot_scans,
        counters.invariant_relation_slot_scans
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_entity_slot_scans,
        counters.invariant_entity_slot_scans
    );
    assert!(counters.relation_cardinality_minimum_certification_relation_slot_scans >= 1);
    assert!(counters.relation_cardinality_minimum_certification_entity_slot_scans >= 2);
    assert!(counters.relation_cardinality_checks >= 1);
}
