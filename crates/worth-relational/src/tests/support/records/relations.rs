use super::*;

pub(crate) fn create_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
) -> RelationId {
    create_relation_with_aspect_label(
        execution,
        runtime,
        source,
        target,
        client_key,
        client_key,
        PartitionId::main(),
    )
}

pub(crate) fn create_relation_in_partition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    partition_id: PartitionId,
) -> RelationId {
    create_relation_with_aspect_label(
        execution,
        runtime,
        source,
        target,
        client_key,
        client_key,
        partition_id,
    )
}

fn create_relation_with_aspect_label(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    label: &str,
    partition_id: PartitionId,
) -> RelationId {
    create_relation_in_partition_on_branch(
        execution,
        runtime,
        source,
        target,
        client_key,
        label,
        partition_id,
        BranchId("main".to_string()),
    )
}

pub(crate) fn create_relation_in_partition_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    label: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> RelationId {
    let fields = relation_fields_for_runtime(runtime, label);
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("relation").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id,
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields,
            },
        ))),
        Allocation::SystemAllocation,
    )
    .unwrap();
    let outcome = txn
        .commit(execution, runtime, Allocation::SystemAllocation)
        .unwrap();
    let relation = changed_relations(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    relation
}

pub(crate) fn create_relation_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
) -> CommitResult {
    let fields = relation_fields_for_runtime(runtime, client_key);
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new("relation").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields,
            },
        ))),
        Allocation::SystemAllocation,
    )
    .unwrap();
    txn.commit(execution, runtime, Allocation::SystemAllocation)
        .unwrap()
}

fn relation_fields_for_runtime(runtime: &RelationalRuntime, label: &str) -> AspectFieldPatch {
    let declares_label = runtime.relation_aspect_plan(KindId(2)).is_some_and(|plan| {
        plan.executable_bindings.iter().any(|binding| {
            binding.aspect_key() == &AspectKey::new("label").expect("valid label aspect key")
        })
    });
    if declares_label {
        relation_label_field_patch(label)
    } else {
        AspectFieldPatch::default()
    }
}
