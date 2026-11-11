use worth_relational::facade::{
    history::BranchId, identity::EntityId, indexes::*, runtime::RelationalRuntime, transactions::*,
};
use worth_testkit_relational::aspect_storage::definition::{
    aspect_field_locator, aspect_key, field_key,
};
use worth_testkit_relational::durable_publication::field_patch;
pub(super) fn name_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "checkpoint-index-tail".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity")
}

pub(super) fn scoped_name_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "scoped-checkpoint-index".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity")
}

pub(super) fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index: DerivedIndexId,
    outcome: &CommitResult,
) -> DerivedIndexGenerationId {
    build_on_branch(execution, runtime, index, outcome, BranchId("main".into()))
}

pub(super) fn build_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index: DerivedIndexId,
    outcome: &CommitResult,
    branch_id: BranchId,
) -> DerivedIndexGenerationId {
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id,
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    built.generations[0].generation_id
}

pub(super) fn fork_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> BranchId {
    super::branch_history_publications::fork(execution, runtime, name);
    BranchId(name.into())
}
pub(super) fn generation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    id: DerivedIndexGenerationId,
) -> Option<DerivedIndexGeneration> {
    runtime
        .index_access()
        .read_generations_for_version(
            execution,
            runtime
                .history()
                .historical_latest_commit()
                .unwrap()
                .version_id,
        )
        .unwrap()
        .into_iter()
        .find(|row| row.generation_id == id)
}
pub(super) fn update_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    let mut txn = super::branch_history_publications::transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: field_patch("name", "name", name),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    txn.commit(execution, runtime).unwrap()
}
