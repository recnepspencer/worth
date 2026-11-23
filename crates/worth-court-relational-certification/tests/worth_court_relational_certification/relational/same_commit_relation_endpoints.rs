use worth_foundational::facade::PortableRecordAspectPatch;
use worth_relational::facade::diagnostics::DiagnosticCode;
use worth_relational::facade::transactions::{
    EntityReference, EntitySpec, MutationIntent, RelationAspectCreateIntent, RelationSpec,
    TransactionCommitError,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, BulkEntityCreateIntent, BulkRelationCreateIntent, CommitResult,
        CreateIntent, CreatedEntityRef, CreatedRelationRef, RecordRef, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, changed_entities},
    durable_publication::{field_patch, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn same_commit_graph_creation_allows_relation_to_target_created_entities() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let source_key = ClientKey::raw("same-commit-source");
    let target_key = ClientKey::raw("same-commit-target");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("same-commit-graph")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key.clone(),
                fields: field_patch("name", "name", "same-commit-source"),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key.clone(),
                fields: field_patch("name", "name", "same-commit-target"),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("same-commit-edge"),
                    source: EntityReference::Created(CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: source_key.clone(),
                    }),
                    target: EntityReference::Created(CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: target_key.clone(),
                    }),
                    fields: AspectFieldPatch::default(),
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(execution, &runtime)
        .expect("same-commit graph creation should succeed");
    let created_entities = changed_entities(&outcome);
    let created_relations = changed_relations(&outcome);

    assert_eq!(created_entities.len(), 2);
    assert_eq!(created_relations.len(), 1);
    assert_eq!(
        outcome.created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("same-commit-edge"),
            source: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key,
            },),
            target: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key,
            },),
        }),
        Some(created_relations[0])
    );
}

#[test]
fn bulk_relation_create_can_target_same_commit_created_entities() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let source_key = ClientKey::raw("bulk-created-source");
    let target_key = ClientKey::raw("bulk-created-target");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-graph")
            .push(MutationIntent::Create(CreateIntent::BulkEntities(
                BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![source_key.clone(), target_key.clone()],
                    field_patches: vec![
                        field_patch("name", "name", "bulk-created-source"),
                        field_patch("name", "name", "bulk-created-target"),
                    ],
                },
            )))
            .push(MutationIntent::Create(CreateIntent::BulkRelations(
                BulkRelationCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_keys: vec![ClientKey::raw("bulk-created-edge")],
                    endpoints: vec![(
                        EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key.clone(),
                        }),
                        EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: target_key.clone(),
                        }),
                    )],
                    field_patches: vec![AspectFieldPatch::default()],
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let outcome = txn
        .commit(execution, &runtime)
        .expect("bulk relation create against created refs should succeed");

    assert_eq!(changed_entities(&outcome).len(), 2);
    let created_relations = changed_relations(&outcome);
    assert_eq!(created_relations.len(), 1);
    assert_eq!(
        outcome.created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("bulk-created-edge"),
            source: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key,
            },),
            target: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key,
            },),
        }),
        Some(created_relations[0])
    );
}

#[test]
fn relation_aspect_create_records_exact_owner_correspondence() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let source_key = ClientKey::raw("aspect-created-source");
    let target_key = ClientKey::raw("aspect-created-target");
    let relation_key = ClientKey::raw("aspect-created-edge");
    let source = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: source_key.clone(),
    };
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: target_key.clone(),
    };

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("aspect-graph")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: source_key,
                fields: field_patch("name", "name", "aspect-created-source"),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: target_key,
                fields: field_patch("name", "name", "aspect-created-target"),
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
        .commit(execution, &runtime)
        .expect("same-commit relation-aspect creation should succeed");
    let created_relation = changed_relations(&outcome)[0];
    assert_eq!(
        outcome.created_relation(&CreatedRelationRef {
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let missing_key = ClientKey::raw("missing-created-endpoint");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("invalid-created-ref").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("invalid-created-edge"),
                source: EntityReference::Created(CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: missing_key.clone(),
                }),
                target: EntityReference::Created(CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: missing_key,
                }),
                fields: AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(execution, &runtime)
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

fn changed_relations(
    outcome: &CommitResult,
) -> Vec<worth_relational::facade::identity::RelationId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Relation(relation) => Some(*relation),
            RecordRef::Entity(_) => None,
        })
        .collect()
}
