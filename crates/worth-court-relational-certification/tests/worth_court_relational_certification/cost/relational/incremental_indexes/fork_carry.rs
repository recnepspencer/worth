//! A fork owns its carried selection before the source's later history can retire.
use super::fixture;
use crate::index_retirement::{aspect_field_locator, create_branch_from_main};
use crate::leased_preparation::ordinary_context::support::{
    aspect_key, field_key, main_basis, string_field_patch,
};
use std::sync::Arc;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    mvcc::RelationalTransactionIntent,
    transactions::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

#[test]
fn first_fork_write_carries_generations_after_parent_history_and_retirement() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture::runtime_with_test_schema();
    let entity = fixture::entities(execution, &runtime, 1)[0];
    let field = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "fork.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .unwrap()
        .index_id;
    let untouched = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "fork.unreached".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
        })
        .unwrap()
        .index_id;
    let basis = main_basis(&runtime);
    let initial = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: basis.identity().branch_id().clone(),
            index_ids: vec![field, untouched],
        },
        &basis,
    );
    assert!(initial.failed_indexes.is_empty());
    let retained = runtime
        .index_access()
        .read_published_generation_for_observation(execution, untouched, &basis.observation())
        .unwrap()
        .unwrap();
    let fork = create_branch_from_main(execution, &runtime, "after-parent-advances");
    drop(basis);
    for step in 0..32 {
        let mut transaction = fixture::transaction(&runtime);
        transaction
            .push_batch(
                update(entity, &format!("parent-{step}")),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let (candidate, _) = fixture::prepare(execution, &runtime, transaction);
        let committed = fixture::publish(execution, &runtime, candidate);
        fixture::release_test_commit_snapshot(&runtime, &committed);
    }
    runtime.retention().run_pass(execution).unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.branch_identity(&fork).unwrap())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            update(entity, "fork-value"),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let (candidate, _) = fixture::prepare(execution, &runtime, transaction);
    let committed = fixture::publish(execution, &runtime, candidate);
    let carried = runtime
        .index_access()
        .read_published_generation_for_commit(execution, untouched, &committed.commit)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&retained, &carried));
    let selected = runtime
        .index_access()
        .read_published_generation_for_commit(execution, field, &committed.commit)
        .unwrap()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.branch_identity(&fork).unwrap())
        .unwrap();
    let cold = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: fork,
            index_ids: vec![field],
        },
        &basis,
    );
    assert!(cold.failed_indexes.is_empty());
    assert_eq!(selected.entries, cold.generations[0].entries);
    fixture::release_test_commit_snapshot(&runtime, &committed);
}
fn update(entity: worth_relational::facade::identity::EntityId, value: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("fork-field").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: string_field_patch(aspect_key("name"), field_key("name"), value),
        }),
    ))
}
