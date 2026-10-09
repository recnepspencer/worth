use super::*;
use crate::capabilities::AspectPlanSource;
use worth_execution::ExecutionAllocationPolicy as Allocation;

pub(crate) fn batch_create(name: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
        CreateIntent::Entity(crate::transactions::data::EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(name),
            fields: crate::tests::support::single_string_aspect_field_patch(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
                name,
            ),
        }),
    ))
}

pub(crate) fn name_field_patch(name: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            AspectKey::new("name").expect("valid name aspect key"),
            worth_foundational::facade::FieldKey::new("name").expect("valid name field key"),
        ),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(name.to_string()),
        ),
    )
}

pub(crate) fn relation_label_field_patch(label: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            AspectKey::new("label").expect("valid relation label aspect key"),
            worth_foundational::facade::FieldKey::new("label").expect("valid label field key"),
        ),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(label.to_string()),
        ),
    )
}

pub(crate) fn create_entity(
    runtime: &RelationalRuntime,
    name: &str,
) -> crate::facade::identity::EntityId {
    let outcome = create_entity_outcome(runtime, name);
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_in_partition(
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
) -> crate::facade::identity::EntityId {
    create_entity_in_partition_on_branch(runtime, name, partition_id, BranchId("main".to_string()))
}

pub(crate) fn create_entity_in_partition_on_branch(
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> crate::facade::identity::EntityId {
    let fields = entity_fields_for_runtime(runtime, name);
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id,
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(name),
                fields,
            }),
        )),
        Allocation::SystemAllocation,
    )
    .unwrap();
    let outcome = txn.commit(runtime, Allocation::SystemAllocation).unwrap();
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_outcome(runtime: &RelationalRuntime, name: &str) -> CommitResult {
    create_entity_outcome_on_branch(runtime, name, BranchId("main".to_string()))
}

pub(crate) fn create_entity_outcome_on_branch(
    runtime: &RelationalRuntime,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    let fields = entity_fields_for_runtime(runtime, name);
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    let mut batch = WorkerIntentBatch::new(format!("batch-{name}"));
    batch
        .intents
        .push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(name),
                fields,
            },
        )));
    txn.push_batch(batch, Allocation::SystemAllocation).unwrap();
    txn.commit(runtime, Allocation::SystemAllocation).unwrap()
}

pub(super) fn entity_fields_for_runtime(
    runtime: &RelationalRuntime,
    name: &str,
) -> AspectFieldPatch {
    let declares_name = runtime.entity_aspect_plan(KindId(1)).is_some_and(|plan| {
        plan.executable_bindings.iter().any(|binding| {
            binding.aspect_key() == &AspectKey::new("name").expect("valid name aspect key")
        })
    });
    if declares_name {
        crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("name"),
            crate::tests::support::field_key("name"),
            name,
        )
    } else {
        AspectFieldPatch::default()
    }
}

pub(crate) fn delete_entity(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
) -> CommitResult {
    delete_entity_on_branch(runtime, entity_id, BranchId("main".to_string()))
}

pub(crate) fn delete_entity_on_branch(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    branch_id: BranchId,
) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        Allocation::SystemAllocation,
    )
    .unwrap();
    txn.commit(runtime, Allocation::SystemAllocation).unwrap()
}

pub(crate) fn delete_relation_on_branch(
    runtime: &RelationalRuntime,
    relation_id: RelationId,
    branch_id: BranchId,
) -> CommitResult {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }),
        )),
        Allocation::SystemAllocation,
    )
    .unwrap();
    txn.commit(runtime, Allocation::SystemAllocation).unwrap()
}

pub(crate) fn update_entity(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) -> CommitResult {
    update_entity_on_branch(runtime, entity_id, name, BranchId("main".to_string()))
}

pub(crate) fn update_entity_and_release_snapshot(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) {
    let outcome = update_entity(runtime, entity_id, name);
    release_test_commit_snapshot(runtime, &outcome);
}

pub(crate) fn update_entity_on_branch(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    try_update_entity_on_branch(runtime, entity_id, name, branch_id).unwrap()
}

pub(crate) fn try_update_entity_on_branch(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
    branch_id: BranchId,
) -> Result<CommitResult, crate::transactions::data::TransactionCommitError> {
    let fields = entity_fields_for_runtime(runtime, name);
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    if !fields.is_empty() {
        txn.push_batch(
            WorkerIntentBatch::new("update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
            )),
            Allocation::SystemAllocation,
        )
        .unwrap();
    }
    txn.commit(runtime, Allocation::SystemAllocation)
}

pub(crate) fn create_relation(
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
) -> RelationId {
    create_relation_with_aspect_label(
        runtime,
        source,
        target,
        client_key,
        client_key,
        PartitionId::main(),
    )
}

pub(crate) fn create_relation_in_partition(
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    partition_id: PartitionId,
) -> RelationId {
    create_relation_with_aspect_label(
        runtime,
        source,
        target,
        client_key,
        client_key,
        partition_id,
    )
}

pub(crate) fn create_relation_with_aspect_label(
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
    client_key: &str,
    label: &str,
    partition_id: PartitionId,
) -> RelationId {
    create_relation_in_partition_on_branch(
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
    let outcome = txn.commit(runtime, Allocation::SystemAllocation).unwrap();
    let relation = changed_relations(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    relation
}

pub(crate) fn create_relation_outcome(
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
    txn.commit(runtime, Allocation::SystemAllocation).unwrap()
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

pub(crate) fn changed_entities(outcome: &CommitResult) -> Vec<crate::facade::identity::EntityId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .collect()
}

pub(crate) fn changed_relations(outcome: &CommitResult) -> Vec<RelationId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Relation(relation_id) => Some(*relation_id),
            RecordRef::Entity(_) => None,
        })
        .collect()
}

pub(crate) fn apply_batches(batches: Vec<WorkerIntentBatch>) -> RelationalRuntime {
    let runtime = runtime_with_test_schema();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    for batch in batches {
        txn.push_batch(batch, Allocation::SystemAllocation).unwrap();
    }
    txn.commit(&runtime, Allocation::SystemAllocation).unwrap();
    runtime
}

pub(crate) fn release_test_commit_snapshot(runtime: &RelationalRuntime, outcome: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("test helper releases the published snapshot it does not return");
}

pub(crate) fn merge_commit_from_branches(
    runtime: &RelationalRuntime,
    target_branch: BranchId,
    merge_parent_branches: Vec<BranchId>,
) -> CommitResult {
    let txn = crate::tests::support::test_owner_begin_merge_transaction(
        runtime,
        target_branch,
        merge_parent_branches,
    );
    txn.commit(runtime, Allocation::SystemAllocation).unwrap()
}
