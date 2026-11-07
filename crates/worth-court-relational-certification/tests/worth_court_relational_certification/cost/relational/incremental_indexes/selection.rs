use super::fixture;
use crate::index_retirement::aspect_field_locator;
use crate::leased_preparation::ordinary_context::support::{
    aspect_key, field_key, main_basis, string_field_patch,
};
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy;
use worth_relational::facade::{
    history::BranchId,
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    transactions::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

#[test]
fn fixed_edit_preserves_unreached_generations_and_is_flat_in_records_history_and_indexes() {
    let baseline = exercise(64, 16, 128);
    fixture::flat("N", baseline, exercise(128, 16, 128));
    fixture::flat("H", exercise(64, 64, 1), exercise(64, 128, 1));
    fixture::flat("non-covering K", baseline, exercise(64, 16, 256));
}
fn exercise(records: usize, history: usize, unrelated_indexes: usize) -> fixture::Work {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = fixture::runtime_with_test_schema();
    let ids = fixture::entities(execution, &runtime, records);
    fixture::relations(execution, &runtime, &[(ids[0], ids[1])]);
    let mut definitions = Vec::new();
    for ordinal in 0..=unrelated_indexes {
        definitions.push(
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("scale-{ordinal}"),
                    branch_scoped: true,
                    kind: if ordinal == 0 {
                        DerivedIndexKind::EntityField {
                            field_locator: aspect_field_locator(
                                aspect_key("name"),
                                field_key("name"),
                            ),
                        }
                    } else {
                        DerivedIndexKind::RelationField {
                            field_locator: aspect_field_locator(
                                aspect_key("label"),
                                field_key("label"),
                            ),
                        }
                    },
                })
                .unwrap(),
        );
    }
    let initial = main_basis(&runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: initial.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: definitions
                .iter()
                .map(|definition| definition.index_id)
                .collect(),
        },
        &initial,
    );
    assert!(built.failed_indexes.is_empty());
    drop(initial);
    let mut history_handles = Vec::new();
    for ordinal in 0..history {
        let transaction = update(&runtime, ids[1], &format!("history-{ordinal}"));
        let (candidate, _) = fixture::prepare(execution, &runtime, transaction);
        let committed = fixture::publish(execution, &runtime, candidate);
        history_handles.push(
            runtime
                .index_access()
                .read_published_generation_for_commit(
                    execution,
                    definitions[0].index_id,
                    &committed.commit,
                )
                .unwrap()
                .unwrap(),
        );
        fixture::release_test_commit_snapshot(&runtime, &committed);
    }
    let before = main_basis(&runtime);
    let prior: Vec<_> = definitions
        .iter()
        .map(|definition| {
            runtime
                .index_access()
                .read_published_generation_for_observation(
                    execution,
                    definition.index_id,
                    &before.observation(),
                )
                .unwrap()
                .unwrap()
        })
        .collect();
    drop(before);
    let (candidate, work) =
        fixture::prepare(execution, &runtime, update(&runtime, ids[0], "after"));
    let committed = fixture::publish(execution, &runtime, candidate);
    let maintained: Vec<_> = definitions
        .iter()
        .map(|definition| {
            runtime
                .index_access()
                .read_published_generation_for_commit(
                    execution,
                    definition.index_id,
                    &committed.commit,
                )
                .unwrap()
                .unwrap()
        })
        .collect();
    for (old, new) in prior.iter().zip(&maintained).skip(1) {
        assert!(Arc::ptr_eq(old, new));
    }
    assert_ne!(prior[0].generation_id, maintained[0].generation_id);
    let basis = main_basis(&runtime);
    let cold = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: committed.commit.branch_id.clone(),
            index_ids: definitions
                .iter()
                .map(|definition| definition.index_id)
                .collect(),
        },
        &basis,
    );
    assert!(cold.failed_indexes.is_empty());
    let mut court = worth_testkit_relational::maintenance::MaintainedCourt::new(
        "native_fields",
        Arc::new(
            built
                .generations
                .into_iter()
                .map(|generation| generation.entries)
                .collect::<Vec<_>>(),
        ),
    );
    court
        .check_against_cold::<()>(
            Arc::new(
                maintained
                    .iter()
                    .map(|generation| generation.entries.clone())
                    .collect::<Vec<_>>(),
            ),
            Arc::new(
                cold.generations
                    .into_iter()
                    .map(|generation| generation.entries)
                    .collect::<Vec<_>>(),
            ),
            |left, right| left == right,
        )
        .unwrap();
    fixture::release_test_commit_snapshot(&runtime, &committed);
    work
}
fn update(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity_id: worth_relational::facade::identity::EntityId,
    value: &str,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = fixture::transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("scale-edit").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: string_field_patch(aspect_key("name"), field_key("name"), value),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
}
