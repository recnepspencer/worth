use super::fixtures::{
    commit_source_min_one_runtime, source_max_one_runtime, target_and_pair_max_one_runtime,
};
use crate::tests::support::*;

#[test]
fn relation_integrity_commit_boundary_rejects_source_cardinality_overflow() {
    let runtime = source_max_one_runtime();
    let source = create_entity(&runtime, "source");
    let target_a = create_entity(&runtime, "target-a");
    let target_b = create_entity(&runtime, "target-b");

    create_relation(&runtime, source, target_a, "a");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("relation").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("b"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target_b),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn.commit(&runtime).unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::RelationCardinalityViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn commit_minimum_rejects_new_entity_without_required_edge() {
    let runtime = commit_source_min_one_runtime();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("orphan").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("orphan"),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
    )
    .expect("stage orphan");
    let error = txn
        .commit(&runtime)
        .expect_err("source minimum must block orphan");
    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code() == DiagnosticCode::RelationCardinalityViolation
                && conflict.detail.contains("source_min_one")
    ));
}

#[test]
fn commit_minimum_uses_local_scope_and_rejects_removed_last_edge() {
    let runtime = commit_source_min_one_runtime();
    let created = crate::transactions::data::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("node"),
    };
    let endpoint = crate::transactions::data::EntityReference::Created(created.clone());
    let relation = crate::transactions::data::CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("self-edge"),
        source: endpoint.clone(),
        target: endpoint.clone(),
    };
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("valid")
            .push(MutationIntent::Create(CreateIntent::Entity(
                crate::transactions::data::EntitySpec {
                    partition_id: created.partition_id,
                    kind_id: created.kind_id,
                    client_key: created.client_key.clone(),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                crate::transactions::data::RelationSpec {
                    partition_id: relation.partition_id,
                    kind_id: relation.kind_id,
                    client_key: relation.client_key.clone(),
                    source: endpoint.clone(),
                    target: endpoint,
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            ))),
    )
    .expect("stage valid node and edge");
    runtime.performance_access().reset_counters();
    let outcome = txn
        .commit(&runtime)
        .expect("local minimum accepts required edge");
    let relation_id = outcome
        .created_relation(&relation)
        .expect("relation binding");
    let counters = runtime.performance_access().counters();
    assert_eq!(
        counters.relation_cardinality_minimum_certification_entity_slot_scans,
        0
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_relation_slot_scans,
        0
    );
    release_test_commit_snapshot(&runtime, &outcome);

    let mut delete = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    delete
        .push_batch(
            WorkerIntentBatch::new("remove-last").push(MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::Delete(
                    crate::transactions::data::DeleteRelationIntent { relation_id },
                ),
            )),
        )
        .expect("stage edge removal");
    let error = delete
        .commit(&runtime)
        .expect_err("last edge underflows source minimum");
    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code() == DiagnosticCode::RelationCardinalityViolation
                && conflict.detail.contains("source_min_one")
    ));
}

#[test]
fn new_target_pair_limit_does_not_scan_existing_source_history() {
    let runtime = target_and_pair_max_one_runtime();
    let source = create_entity(&runtime, "source");
    let prior_target = create_entity(&runtime, "prior-target");
    create_relation(&runtime, source, prior_target, "prior-edge");
    let created_target = crate::transactions::data::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("new-target"),
    };
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("new-pair")
            .push(MutationIntent::Create(CreateIntent::Entity(
                crate::transactions::data::EntitySpec {
                    partition_id: created_target.partition_id,
                    kind_id: created_target.kind_id,
                    client_key: created_target.client_key.clone(),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                crate::transactions::data::RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("new-edge"),
                    source: crate::transactions::data::EntityReference::Existing(source),
                    target: crate::transactions::data::EntityReference::Created(created_target),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            ))),
    )
    .expect("stage fresh pair");
    runtime.performance_access().reset_counters();
    let outcome = txn.commit(&runtime).expect("fresh pair remains unique");
    assert_eq!(
        runtime
            .performance_access()
            .counters()
            .relation_uniqueness_candidates_scanned,
        0,
        "a newly created target cannot have a committed pair with this source"
    );
    release_test_commit_snapshot(&runtime, &outcome);

    let mut duplicate_target =
        crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let new_source = crate::transactions::data::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("new-source"),
    };
    duplicate_target
        .push_batch(
            WorkerIntentBatch::new("duplicate-target")
                .push(MutationIntent::Create(CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: new_source.partition_id,
                        kind_id: new_source.kind_id,
                        client_key: new_source.client_key.clone(),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    },
                )))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    crate::transactions::data::RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: crate::symbols::data::ClientKey::raw("second-owner"),
                        source: crate::transactions::data::EntityReference::Created(new_source),
                        target: crate::transactions::data::EntityReference::Existing(prior_target),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    },
                ))),
        )
        .expect("stage second owner");
    let error = duplicate_target
        .commit(&runtime)
        .expect_err("existing target cardinality must still be counted");
    assert!(
        matches!(
            error,
            TransactionCommitError::Conflict { error: ref conflict, .. }
                if conflict.code() == DiagnosticCode::RelationCardinalityViolation
                    && conflict.detail.contains("target_and_pair_max_one")
        ),
        "unexpected duplicate denial: {error:?}"
    );
}

#[test]
fn deleting_endpoint_cannot_hide_survivor_minimum_underflow() {
    let runtime = commit_source_min_one_runtime();
    let source = crate::transactions::data::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("source"),
    };
    let removed = crate::transactions::data::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("removed"),
    };
    let create_entity = |created: &crate::transactions::data::CreatedEntityRef| {
        MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))
    };
    let create_relation = |key, from, to| {
        MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(key),
                source: crate::transactions::data::EntityReference::Created(from),
                target: crate::transactions::data::EntityReference::Created(to),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))
    };
    let mut create = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("valid-pair")
                .push(create_entity(&source))
                .push(create_entity(&removed))
                .push(create_relation(
                    "source-to-removed",
                    source.clone(),
                    removed.clone(),
                ))
                .push(create_relation(
                    "removed-self",
                    removed.clone(),
                    removed.clone(),
                )),
        )
        .expect("stage valid initial cardinality");
    let outcome = create.commit(&runtime).expect("both sources have one edge");
    let removed_id = outcome
        .created_entity(&removed)
        .expect("removed entity binding");
    release_test_commit_snapshot(&runtime, &outcome);

    let mut delete = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    delete
        .push_batch(
            WorkerIntentBatch::new("delete-endpoint").push(MutationIntent::Entity(
                crate::transactions::data::EntityMutationIntent::Delete(
                    crate::transactions::data::DeleteEntityIntent {
                        entity_id: removed_id,
                    },
                ),
            )),
        )
        .expect("stage endpoint deletion");
    let error = delete
        .commit(&runtime)
        .expect_err("survivor loses its last edge");
    assert!(
        matches!(
            error,
            TransactionCommitError::Conflict { error: ref conflict, .. }
                if conflict.code() == DiagnosticCode::RelationCardinalityViolation
                    && conflict.detail.contains("source_min_one")
        ),
        "unexpected deletion denial: {error:?}"
    );
}
