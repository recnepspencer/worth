use crate::facade::transactions::MutationIntent;
use crate::tests::support::*;

#[test]
fn fieldless_entity_create_commits_with_absent_authoritative_aspect_state() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .build();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("opaque").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("opaque"),
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
        .unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let entity_id = read.entities().first().unwrap().entity_id;

    assert_eq!(runtime.entity_history_len_for_test(entity_id), 1);
    assert!(read.entities()[0].authoritative_aspect_state.is_none());
}

#[test]
fn authoritative_field_patches_are_order_independent_in_patch_output() {
    let order_independent_schema = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("a"),
                crate::tests::support::field_key("a"),
            ),
            entity_field_aspect(
                crate::tests::support::aspect_key("b"),
                crate::tests::support::field_key("b"),
            ),
            lifecycle_aspect(),
        ],
        relation_aspects: vec![
            lifecycle_aspect(),
            relation_source_aspect(),
            relation_target_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let left_runtime = RelationalRuntimeApi::builder()
        .schema_registry(order_independent_schema.clone())
        .build();
    let right_runtime = RelationalRuntimeApi::builder()
        .schema_registry(order_independent_schema)
        .build();

    let mut left_txn = crate::tests::support::test_owner_begin_transaction_for_main(&left_runtime);
    left_txn
        .push_batch(
            WorkerIntentBatch::new("left-field-patch").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("entity"),
                    fields: crate::tests::support::string_aspect_field_patch([
                        (
                            crate::tests::support::aspect_key("b"),
                            crate::tests::support::field_key("b"),
                            "second",
                        ),
                        (
                            crate::tests::support::aspect_key("a"),
                            crate::tests::support::field_key("a"),
                            "first",
                        ),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    left_txn
        .commit(
            &left_runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let mut right_txn =
        crate::tests::support::test_owner_begin_transaction_for_main(&right_runtime);
    right_txn
        .push_batch(
            WorkerIntentBatch::new("right-field-patch").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("entity"),
                    fields: crate::tests::support::string_aspect_field_patch([
                        (
                            crate::tests::support::aspect_key("a"),
                            crate::tests::support::field_key("a"),
                            "first",
                        ),
                        (
                            crate::tests::support::aspect_key("b"),
                            crate::tests::support::field_key("b"),
                            "second",
                        ),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    right_txn
        .commit(
            &right_runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert_eq!(
        left_runtime.publication().artifacts().latest_patch(),
        right_runtime.publication().artifacts().latest_patch()
    );
}
