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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> crate::facade::identity::EntityId {
    let outcome = create_entity_outcome(execution, runtime, name);
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_in_partition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
) -> crate::facade::identity::EntityId {
    create_entity_in_partition_on_branch(
        execution,
        runtime,
        name,
        partition_id,
        BranchId("main".to_string()),
    )
}

pub(crate) fn create_entity_in_partition_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> crate::facade::identity::EntityId {
    let outcome = commit_entity(execution, runtime, name, partition_id, branch_id);
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> CommitResult {
    create_entity_outcome_on_branch(execution, runtime, name, BranchId("main".to_string()))
}

pub(crate) fn create_entity_outcome_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    commit_entity(execution, runtime, name, PartitionId::main(), branch_id)
}

fn commit_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> CommitResult {
    let fields = entity_fields_for_runtime(runtime, name);
    let batch = WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
        CreateIntent::Entity(crate::transactions::data::EntitySpec {
            partition_id,
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(name),
            fields,
        }),
    ));
    commit_batches(execution, runtime, branch_id, [batch]).unwrap()
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
) -> CommitResult {
    delete_entity_on_branch(execution, runtime, entity_id, BranchId("main".to_string()))
}

pub(crate) fn delete_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    branch_id: BranchId,
) -> CommitResult {
    let batch = WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
    ));
    commit_batches(execution, runtime, branch_id, [batch]).unwrap()
}

pub(crate) fn delete_relation_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    relation_id: RelationId,
    branch_id: BranchId,
) -> CommitResult {
    let batch = WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
        RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }),
    ));
    commit_batches(execution, runtime, branch_id, [batch]).unwrap()
}

pub(crate) fn update_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) -> CommitResult {
    update_entity_on_branch(
        execution,
        runtime,
        entity_id,
        name,
        BranchId("main".to_string()),
    )
}

pub(crate) fn update_entity_and_release_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) {
    let outcome = update_entity(execution, runtime, entity_id, name);
    release_test_commit_snapshot(runtime, &outcome);
}

pub(crate) fn update_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    try_update_entity_on_branch(execution, runtime, entity_id, name, branch_id).unwrap()
}

pub(crate) fn try_update_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
    branch_id: BranchId,
) -> Result<CommitResult, crate::transactions::data::TransactionCommitError> {
    let fields = entity_fields_for_runtime(runtime, name);
    let batch = (!fields.is_empty()).then(|| {
        WorkerIntentBatch::new("update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        ))
    });
    commit_batches(execution, runtime, branch_id, batch)
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

pub(crate) fn apply_batches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    batches: Vec<WorkerIntentBatch>,
) -> RelationalRuntime {
    let runtime = runtime_with_test_schema();
    let main = runtime.config().history.main_branch.clone();
    commit_batches(execution, &runtime, main, batches).unwrap();
    runtime
}

fn commit_batches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batches: impl IntoIterator<Item = WorkerIntentBatch>,
) -> Result<CommitResult, TransactionCommitError> {
    let mut txn = test_owner_begin_transaction_for_branch(runtime, branch_id);
    for batch in batches {
        txn.push_batch(batch, Allocation::SystemAllocation).unwrap();
    }
    txn.commit(execution, runtime)
}

pub(crate) fn release_test_commit_snapshot(runtime: &RelationalRuntime, outcome: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("test helper releases the published snapshot it does not return");
}

#[path = "records/relations.rs"]
mod relations;
pub(crate) use relations::{
    create_relation, create_relation_in_partition, create_relation_in_partition_on_branch,
    create_relation_outcome,
};

/// An actual Memory owner, its acknowledged record, and the original execution.
/// The fixture releases its initial result snapshot before lending the world.
pub(crate) struct MemoryRecordPublicationWorld<'host, 'scope> {
    pub(crate) runtime: RelationalRuntime,
    pub(crate) entity: crate::identity::data::EntityId,
    pub(crate) execution: worth_execution::ExecutionRequest<'host, 'scope>,
}
pub(crate) fn with_memory_record_publication<R>(
    registry: RelationalSchemaRegistry,
    name: &str,
    observe: impl for<'host, 'scope> FnOnce(MemoryRecordPublicationWorld<'host, 'scope>) -> R,
) -> R {
    let original = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&original);
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("valid Memory record publication world");
    let entity = create_entity(&execution, &runtime, name);
    observe(MemoryRecordPublicationWorld {
        runtime,
        entity,
        execution,
    })
}
