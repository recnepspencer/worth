use super::*;

pub(crate) fn create_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
) -> RelationId {
    create_relation_in_partition(execution, runtime, source, target, client_key, PartitionId::main())
}

pub(crate) fn create_relation_in_partition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    partition_id: PartitionId,
) -> RelationId {
    create_relation_in_partition_on_branch(
        execution, runtime, source, target, client_key, client_key,
        partition_id, BranchId("main".to_string()),
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
    let outcome = commit_relation(
        execution,
        runtime,
        source,
        target,
        client_key,
        label,
        partition_id,
        branch_id,
    );
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
    commit_relation(
        execution,
        runtime,
        source,
        target,
        client_key,
        client_key,
        PartitionId::main(),
        runtime.config().history.main_branch.clone(),
    )
}

fn commit_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    label: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> CommitResult {
    let fields = relation_fields_for_runtime(runtime, label);
    let batch = WorkerIntentBatch::new("relation").push(MutationIntent::Create(
        CreateIntent::Relation(crate::transactions::data::RelationSpec {
            partition_id,
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw(client_key),
            source: crate::transactions::data::EntityReference::Existing(source),
            target: crate::transactions::data::EntityReference::Existing(target),
            fields,
        }),
    ));
    commit_batches(execution, runtime, branch_id, [batch]).unwrap()
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
