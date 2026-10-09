use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::transactions::{CommitConflict, TransactionCommitError};
use crate::tests::support::*;

#[test]
fn bulk_create_entities_match_equivalent_singular_creates() {
    let bulk_runtime = runtime_with_test_schema();
    let mut bulk_txn = crate::tests::support::test_owner_begin_transaction_for_main(&bulk_runtime);
    bulk_txn
        .push_batch(
            WorkerIntentBatch::new("bulk").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![
                        crate::symbols::data::ClientKey::raw("a"),
                        crate::symbols::data::ClientKey::raw("b"),
                    ],
                    field_patches: vec![
                        single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "a",
                        ),
                        single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "b",
                        ),
                    ],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let bulk_outcome = bulk_txn
        .commit(
            &bulk_runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let singular_runtime = apply_batches(vec![batch_create("a"), batch_create("b")]);
    let bulk_read = bulk_runtime
        .read_truth()
        .read_snapshot(&bulk_outcome.snapshot)
        .unwrap();
    let singular_read = singular_runtime
        .read_truth()
        .read_snapshot(
            &singular_runtime
                .publication()
                .latest_bundle()
                .unwrap()
                .snapshot,
        )
        .unwrap();

    assert_eq!(bulk_outcome.changed_records.len(), 2);
    assert_eq!(bulk_read.entities().len(), singular_read.entities().len());
    assert_eq!(
        bulk_read
            .entities()
            .iter()
            .map(read_entity_name)
            .collect::<Vec<_>>(),
        singular_read
            .entities()
            .iter()
            .map(read_entity_name)
            .collect::<Vec<_>>()
    );
}

#[test]
fn staged_parallel_bulk_entity_import_matches_serial_reference() {
    fn bulk_commit(
        leased: bool,
    ) -> (
        crate::facade::transactions::CommitResult,
        Vec<Option<String>>,
    ) {
        let runtime = runtime_with_test_schema();
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("bulk").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![
                        crate::symbols::data::ClientKey::raw("a"),
                        crate::symbols::data::ClientKey::raw("b"),
                        crate::symbols::data::ClientKey::raw("c"),
                    ],
                    field_patches: vec![
                        single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "a",
                        ),
                        single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "b",
                        ),
                        single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "c",
                        ),
                    ],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        let outcome = if leased {
            runtime.commit_branch_transaction_with_lease(txn, &test_execution_lease())
        } else {
            txn.commit(
                &runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        }
        .unwrap();
        let read = runtime
            .read_truth()
            .read_snapshot(&outcome.snapshot)
            .unwrap();
        let names = read
            .entities()
            .iter()
            .map(read_entity_name)
            .collect::<Vec<_>>();
        (outcome, names)
    }

    let (serial_outcome, serial_names) = bulk_commit(false);
    let (staged_outcome, staged_names) = bulk_commit(true);

    assert_eq!(
        staged_outcome.changed_records.len(),
        serial_outcome.changed_records.len()
    );
    assert_eq!(staged_names, serial_names);
    assert_eq!(staged_outcome.patch(), serial_outcome.patch());
}

#[test]
fn cross_context_relations_preserve_partitioned_endpoints() {
    let runtime = runtime_with_test_schema();
    let source = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let target = create_entity_in_partition(&runtime, "right", PartitionId(11));
    let relation =
        create_relation_in_partition(&runtime, source, target, "bridge", PartitionId(29));
    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime.read_truth().read_snapshot(&snapshot).unwrap();
    let relation_record = read.get_relation(relation).unwrap();

    assert_eq!(relation.partition_id, PartitionId(29));
    assert_eq!(relation_record.source.partition_id, PartitionId(7));
    assert_eq!(relation_record.target.partition_id, PartitionId(11));
    assert_eq!(relation_record.relation_id.partition_id, PartitionId(29));
}

#[test]
fn cross_context_relations_respect_relation_kind_policy() {
    let schema_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::SchemaControlled,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema_registry)
        .cross_context_policy(CrossContextPolicy::SchemaControlled)
        .build();
    let source = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let target = create_entity_in_partition(&runtime, "right", PartitionId(11));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("forbidden-cross-context").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId(29),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("bridge"),
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

    assert!(matches!(
        error,
        TransactionCommitError::Conflict {
            error: CommitConflict {
                code: DiagnosticCode::InvalidRelationEndpoint,
                ..
            },
            ..
        }
    ));
}

#[test]
fn partition_registry_and_stats_expose_partition_owned_state() {
    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let left = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right", PartitionId(11));
    let _ = create_relation_in_partition(&runtime, left, right, "bridge", PartitionId(29));

    let partition_ids = runtime.storage_access().partition_ids();
    let stats = runtime.storage_access().partition_storage_stats();

    assert_eq!(
        partition_ids,
        vec![PartitionId(7), PartitionId(11), PartitionId(29)]
    );
    assert_eq!(stats.len(), 3);
    assert_eq!(
        stats
            .iter()
            .find(|entry| entry.partition_id == PartitionId(7))
            .unwrap()
            .live_entities,
        1
    );
    assert_eq!(
        stats
            .iter()
            .find(|entry| entry.partition_id == PartitionId(11))
            .unwrap()
            .live_entities,
        1
    );
    assert_eq!(
        stats
            .iter()
            .find(|entry| entry.partition_id == PartitionId(29))
            .unwrap()
            .live_relations,
        1
    );
}
