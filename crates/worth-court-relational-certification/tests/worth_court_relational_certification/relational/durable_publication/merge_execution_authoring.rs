//! Ordinary branch authoring of declared aspect values and partitioned relations.
use super::branch_history_publications::{changed_relations, transaction};
use super::merge_execution_observation::read_entity_aspect_field_display;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, KindId, PartitionId, RelationId},
    runtime::RelationalRuntime,
    transactions::{
        planned_single_field_locator, AspectFieldPatch, CreateIntent, EntityMutationIntent,
        EntityReference, EntitySpec, MutationIntent, RelationSpec, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::{changed_entities, release_test_commit_snapshot, string_aspect_value},
    },
    durable_publication::field_patch,
};

pub(super) fn aspect_field_patch_from_values(
    fields: impl IntoIterator<Item = (AspectKey, FieldKey, AspectValue)>,
) -> AspectFieldPatch {
    AspectFieldPatch::from(
        fields
            .into_iter()
            .map(|(aspect, field, value)| (planned_single_field_locator(aspect, field), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

pub(super) fn single_string_aspect_field_patch(
    aspect: AspectKey,
    field: FieldKey,
    value: &str,
) -> AspectFieldPatch {
    aspect_field_patch_from_values([(aspect, field, string_aspect_value(value))])
}

pub(super) fn i64_counter_value(value: i64) -> AspectValue {
    AspectValue::Int64(value)
}

/// `fields`, plus the stable name identity when the caller did not author one.
fn aspect_fields_with_identity_name(
    client_key: &str,
    fields: AspectFieldPatch,
) -> AspectFieldPatch {
    let mut targets = fields
        .iter()
        .map(|(target, value)| (target.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    targets
        .entry(planned_single_field_locator(
            aspect_key("name"),
            field_key("name"),
        ))
        .or_insert_with(|| string_aspect_value(client_key));
    AspectFieldPatch::new(targets)
}

pub(super) fn create_entity_with_aspect_fields(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    client_key: &str,
    fields: AspectFieldPatch,
) -> EntityId {
    create_entity_with_aspect_fields_on_branch(
        execution,
        runtime,
        client_key,
        fields,
        BranchId("main".to_string()),
    )
}

fn create_entity_with_aspect_fields_on_branch(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    client_key: &str,
    fields: AspectFieldPatch,
    branch_id: BranchId,
) -> EntityId {
    let mut txn = transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new(format!("create-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: client_key.into(),
                fields: aspect_fields_with_identity_name(client_key, fields),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_entities(&txn.commit(execution, runtime).unwrap())[0]
}

pub(super) fn update_entity_aspect_fields_on_branch(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    fields: AspectFieldPatch,
    branch_id: BranchId,
) {
    let stable_name = read_entity_aspect_field_display(
        execution,
        runtime,
        &branch_id,
        entity_id,
        aspect_key("name"),
        field_key("name"),
    );
    let mut txn = transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("update-aspect-fields").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: aspect_fields_with_identity_name(&stable_name, fields),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, runtime).unwrap();
}

/// A labelled relation authored on `branch_id`, for schemas declaring `label`.
pub(super) fn create_relation_in_partition_on_branch(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    client_key: &str,
    label: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> RelationId {
    let mut txn = transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("relation").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id,
                kind_id: KindId(2),
                client_key: client_key.into(),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: field_patch("label", "label", label),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let outcome = txn.commit(execution, runtime).unwrap();
    let relation = changed_relations(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    relation
}
