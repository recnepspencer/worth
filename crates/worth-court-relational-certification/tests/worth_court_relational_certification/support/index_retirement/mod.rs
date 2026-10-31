//! Ordinary index publication actions over the shared aspect-storage schema.
pub(crate) use worth_relational::facade::{
    history::BranchId, identity::*, indexes::*, runtime::RelationalRuntime,
    transactions::CommitResult,
};
pub(crate) use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
pub(crate) use worth_testkit_relational::aspect_storage::{
    aspect_field_locator, aspect_key, field_key,
};

pub(crate) fn runtime_with_test_schema() -> RelationalRuntime {
    worth_testkit_relational::aspect_storage::AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_runtime()
}
pub(crate) fn create_entity_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> CommitResult {
    crate::leased_preparation::ordinary_context::support::create_entity(execution, runtime, name).0
}
pub(crate) fn create_branch_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> BranchId {
    let branch = BranchId(name.into());
    let (_, basis) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    runtime
        .fork_branch(execution, branch.clone(), basis)
        .unwrap();
    branch
}
pub(crate) fn name_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
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
        .unwrap()
}
pub(crate) fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index: DerivedIndexId,
    outcome: &CommitResult,
) -> DerivedIndexGenerationId {
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    built.generations[0].generation_id
}
pub(crate) fn generation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index: DerivedIndexId,
    outcome: &CommitResult,
) -> Option<DerivedIndexGenerationId> {
    runtime
        .index_access()
        .read_published_generation_for_commit(execution, index, &outcome.commit)
        .unwrap()
        .map(|generation| generation.generation_id)
}
