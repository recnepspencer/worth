use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::transactions::{
    EntityReference, EntitySpec, MutationIntent, RelationAspectCreateIntent, RelationSpec,
    TransactionCommitError,
};
use crate::tests::support::*;
use worth_foundational::facade::PortableRecordAspectPatch;

#[test]
fn same_commit_graph_creation_allows_relation_to_target_created_entities() {
    let runtime = runtime_with_test_schema();
    let source_key = crate::symbols::data::ClientKey::raw("same-commit-source");
    let target_key = crate::symbols::data::ClientKey::raw("same-commit-target");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("same-commit-graph")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key.clone(),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "same-commit-source",
                ),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key.clone(),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "same-commit-target",
                ),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("same-commit-edge"),
                    source: crate::facade::transactions::EntityReference::Created(
                        crate::facade::transactions::CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key.clone(),
                        },
                    ),
                    target: crate::facade::transactions::EntityReference::Created(
                        crate::facade::transactions::CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: target_key.clone(),
                        },
                    ),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("same-commit graph creation should succeed");
    let created_entities = changed_entities(&outcome);
    let created_relations = changed_relations(&outcome);

    assert_eq!(created_entities.len(), 2);
    assert_eq!(created_relations.len(), 1);
    assert_eq!(
        outcome.created_relation(&crate::facade::transactions::CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw("same-commit-edge"),
            source: crate::facade::transactions::EntityReference::Created(
                crate::facade::transactions::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: source_key,
                },
            ),
            target: crate::facade::transactions::EntityReference::Created(
                crate::facade::transactions::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: target_key,
                },
            ),
        }),
        Some(created_relations[0])
    );
}

#[test]
fn bulk_relation_create_can_target_same_commit_created_entities() {
    let runtime = runtime_with_test_schema();
    let source_key = crate::symbols::data::ClientKey::raw("bulk-created-source");
    let target_key = crate::symbols::data::ClientKey::raw("bulk-created-target");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-graph")
            .push(MutationIntent::Create(CreateIntent::BulkEntities(
                crate::facade::transactions::BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![source_key.clone(), target_key.clone()],
                    field_patches: vec![
                        crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "bulk-created-source",
                        ),
                        crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "bulk-created-target",
                        ),
                    ],
                },
            )))
            .push(MutationIntent::Create(CreateIntent::BulkRelations(
                crate::facade::transactions::BulkRelationCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_keys: vec![crate::symbols::data::ClientKey::raw("bulk-created-edge")],
                    endpoints: vec![(
                        crate::facade::transactions::EntityReference::Created(
                            crate::facade::transactions::CreatedEntityRef {
                                partition_id: PartitionId::main(),
                                kind_id: KindId(1),
                                client_key: source_key.clone(),
                            },
                        ),
                        crate::facade::transactions::EntityReference::Created(
                            crate::facade::transactions::CreatedEntityRef {
                                partition_id: PartitionId::main(),
                                kind_id: KindId(1),
                                client_key: target_key.clone(),
                            },
                        ),
                    )],
                    field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("bulk relation create against created refs should succeed");

    assert_eq!(changed_entities(&outcome).len(), 2);
    let created_relations = changed_relations(&outcome);
    assert_eq!(created_relations.len(), 1);
    assert_eq!(
        outcome.created_relation(&crate::facade::transactions::CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw("bulk-created-edge"),
            source: crate::facade::transactions::EntityReference::Created(
                crate::facade::transactions::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: source_key,
                },
            ),
            target: crate::facade::transactions::EntityReference::Created(
                crate::facade::transactions::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: target_key,
                },
            ),
        }),
        Some(created_relations[0])
    );
}

#[test]
fn relation_aspect_create_records_exact_owner_correspondence() {
    let runtime = runtime_with_test_schema();
    let source_key = crate::symbols::data::ClientKey::raw("aspect-created-source");
    let target_key = crate::symbols::data::ClientKey::raw("aspect-created-target");
    let relation_key = crate::symbols::data::ClientKey::raw("aspect-created-edge");
    let source = crate::facade::transactions::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: source_key.clone(),
    };
    let target = crate::facade::transactions::CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: target_key.clone(),
    };

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("aspect-graph")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "aspect-created-source",
                ),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "aspect-created-target",
                ),
            })))
            .push(MutationIntent::Create(CreateIntent::RelationAspects(
                RelationAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: relation_key.clone(),
                    source: EntityReference::Created(source.clone()),
                    target: EntityReference::Created(target.clone()),
                    aspect_patch: PortableRecordAspectPatch::new([]),
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("same-commit relation-aspect creation should succeed");
    let created_relation = changed_relations(&outcome)[0];
    assert_eq!(
        outcome.created_relation(&crate::facade::transactions::CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: relation_key,
            source: EntityReference::Created(source),
            target: EntityReference::Created(target),
        }),
        Some(created_relation)
    );
}

#[test]
fn relation_create_rejects_created_entity_refs_missing_from_same_commit() {
    let runtime = runtime_with_test_schema();
    let missing_key = crate::symbols::data::ClientKey::raw("missing-created-endpoint");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("invalid-created-ref").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("invalid-created-edge"),
                source: crate::facade::transactions::EntityReference::Created(
                    crate::facade::transactions::CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: missing_key.clone(),
                    },
                ),
                target: crate::facade::transactions::EntityReference::Created(
                    crate::facade::transactions::CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: missing_key,
                    },
                ),
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
        .expect_err("missing created ref should fail closed");
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::InvalidRelationEndpoint);
        }
        other => panic!(
            "expected invalid relation endpoint conflict, got {:?}",
            other
        ),
    }
}
