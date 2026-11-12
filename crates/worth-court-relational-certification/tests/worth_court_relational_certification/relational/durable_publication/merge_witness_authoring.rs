//! Actual branch transactions for schema witness and collaboration histories.
use super::branch_history_publications::{
    changed_relations, create_entity, create_with_fields, fork, transaction,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId, identity::*, merge::*, runtime::RelationalRuntime, transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::{changed_entities, release_test_commit_snapshot},
    },
    durable_publication::field_patch,
};
fn string_aspect_field_patch<'a>(
    fields: impl IntoIterator<
        Item = (
            worth_foundational::facade::AspectKey,
            worth_foundational::facade::FieldKey,
            &'a str,
        ),
    >,
) -> AspectFieldPatch {
    AspectFieldPatch::from(
        fields
            .into_iter()
            .map(|(aspect, field, value)| {
                (
                    planned_single_field_locator(aspect, field),
                    worth_foundational::facade::AspectValue::String(value.into()),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>(),
    )
}
pub(super) fn create_named_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    client_key: &str,
    name: &str,
    status: Option<&str>,
    branch: &str,
) {
    let mut fields = vec![(aspect_key("name"), field_key("name"), name)];
    if let Some(status) = status {
        fields.push((aspect_key("status"), field_key("status"), status));
    }
    let mut txn = transaction(runtime, &BranchId(branch.into()));
    txn.push_batch(
        WorkerIntentBatch::new(format!("seed-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(worth_relational::facade::transactions::EntitySpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: client_key.into(),
                fields: string_aspect_field_patch(fields),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, runtime).expect("seed entity");
}
pub(super) fn update_entity_status_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: worth_relational::facade::identity::EntityId,
    status: &str,
    branch: &str,
) {
    let mut txn = transaction(runtime, &BranchId(branch.into()));
    txn.push_batch(
        worth_relational::facade::transactions::WorkerIntentBatch::new(format!("status-{branch}"))
            .push(
                worth_relational::facade::transactions::MutationIntent::Entity(
                    worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                        worth_relational::facade::transactions::UpdateEntityFieldsIntent {
                            entity_id,
                            fields: field_patch("status", "status", status),
                        },
                    ),
                ),
            ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, runtime).expect("update status");
}
pub(super) fn create_empty_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    let result = create_with_fields(
        request,
        runtime,
        key,
        &BranchId("main".into()),
        AspectFieldPatch::default(),
    );
    let entity = changed_entities(&result)[0];
    release_test_commit_snapshot(runtime, &result);
    entity
}
pub(super) fn create_relation_on_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: BranchId,
    spec: RelationSpec,
) -> RelationId {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(
        WorkerIntentBatch::new("relation")
            .push(MutationIntent::Create(CreateIntent::Relation(spec))),
        request.allocation_policy(),
    )
    .unwrap();
    let result = txn.commit(request, runtime).unwrap();
    let relation = changed_relations(&result)[0];
    release_test_commit_snapshot(runtime, &result);
    relation
}
pub(super) fn create_relation(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> RelationId {
    create_relation_on_branch(
        request,
        runtime,
        BranchId("main".into()),
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: key.into(),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields: field_patch("label", "label", key),
        },
    )
}
pub(super) fn prepared_collaboration(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    alternate: bool,
) -> PreparedMergeExecution {
    let shared = create_entity(execution, runtime, "shared");
    fork(execution, runtime, "feature");
    if alternate {
        fork(execution, runtime, "alt");
    }
    update_entity_status_on_branch(execution, runtime, shared, "inactive", "main");
    update_entity_status_on_branch(execution, runtime, shared, "active", "feature");
    if alternate {
        update_entity_status_on_branch(execution, runtime, shared, "pending", "alt");
    }
    runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(super::merge_proof_journey::merge_request())
                .unwrap(),
        )
        .unwrap()
}
