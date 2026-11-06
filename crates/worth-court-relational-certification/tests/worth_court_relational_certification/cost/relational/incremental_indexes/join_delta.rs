use super::fixture;
use crate::index_retirement::{
    BranchId, DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::leased_preparation::ordinary_context::support::{
    aspect_key, field_key, main_basis, string_field_patch,
};
use worth_execution::ExecutionAllocationPolicy;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    indexes::{RelationJoinDefinition, RelationJoinLeg, RelationJoinSharedEndpoint},
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntityReference, MutationIntent, RelationMutationIntent, RelationSpec,
        UpdateRelationEndpointsIntent, WorkerIntentBatch,
    },
};

#[test]
fn one_join_leg_edit_is_flat_in_unchanged_hub_degree_and_retains_unpaired_legs() {
    fixture::flat("unchanged hub degree", exercise(256), exercise(512));
}
fn exercise(degree: usize) -> fixture::Work {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = fixture::runtime_with_test_schema();
    let ids = fixture::entities(execution, &runtime, degree + 5);
    let hub = ids[0];
    let left = &ids[1..=degree];
    let right = ids[degree + 1];
    let replacement = ids[degree + 2];
    let unpaired_hub = ids[degree + 3];
    let unpaired_right = ids[degree + 4];
    let mut edges: Vec<_> = left.iter().map(|left| (*left, hub)).collect();
    edges.extend([(hub, right), (replacement, unpaired_hub)]);
    fixture::relations(execution, &runtime, &edges);
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "asymmetric-join".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
        })
        .unwrap();
    let basis = main_basis(&runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    drop(basis);
    let relation = fixture::relation_between(execution, &runtime, left[0], hub);
    let mut transaction = fixture::transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("one-leg-retarget").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(replacement),
                    target: EntityReference::Existing(hub),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let (candidate, work) = fixture::prepare(execution, &runtime, transaction);
    let committed = fixture::publish(execution, &runtime, candidate);
    assert_cold(execution, &runtime, definition.index_id, &committed);
    fixture::release_test_commit_snapshot(&runtime, &committed);
    let mut transaction = fixture::transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("pair-previously-unpaired-leg").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("new-opposite-leg"),
                    source: EntityReference::Existing(unpaired_hub),
                    target: EntityReference::Existing(unpaired_right),
                    fields: string_field_patch(
                        aspect_key("label"),
                        field_key("label"),
                        "new-opposite-leg",
                    ),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let (candidate, _) = fixture::prepare(execution, &runtime, transaction);
    let committed = fixture::publish(execution, &runtime, candidate);
    let selected = runtime
        .index_access()
        .read_published_generation_for_commit(execution, definition.index_id, &committed.commit)
        .unwrap()
        .unwrap();
    let worth_relational::facade::indexes::DerivedIndexEntries::RelationJoin(rows) =
        &selected.entries
    else {
        panic!("join generation")
    };
    let key = worth_relational::facade::indexes::RelationJoinKey::new(replacement, unpaired_right);
    assert!(rows
        .get(&key)
        .unwrap()
        .iter()
        .any(|row| row.shared_entity_id() == unpaired_hub));
    assert_cold(execution, &runtime, definition.index_id, &committed);
    fixture::release_test_commit_snapshot(&runtime, &committed);
    work
}
fn assert_cold(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    index: DerivedIndexId,
    committed: &worth_relational::facade::transactions::CommitResult,
) {
    let selected = runtime
        .index_access()
        .read_published_generation_for_commit(execution, index, &committed.commit)
        .unwrap()
        .unwrap();
    let cold = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: committed.commit.branch_id.clone(),
            index_ids: vec![index],
        },
        &main_basis(runtime),
    );
    assert!(cold.failed_indexes.is_empty());
    assert_eq!(selected.entries, cold.generations[0].entries);
}
