//! Shipping generation reads and bounded candidates survive real source recovery.
use super::branch_history_publications::changed_relations;
use super::checkpoint_recovery_observation::snapshot_for_branch;
use super::*;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    history::BranchId,
    identity::EntityId,
    indexes::*,
    runtime::RelationalRuntime,
    transactions::{CommitResult, RelationSpec},
};
use worth_testkit_relational::aspect_storage::{
    definition::{aspect_field_locator, aspect_key, field_key},
    publications::changed_entities,
};
pub(super) fn create_relation_outcome(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> CommitResult {
    let mut transaction =
        super::branch_history_publications::transaction(runtime, &BranchId("main".into()));
    transaction
        .push_batch(
            WorkerIntentBatch::new("relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: key.into(),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: field_patch("label", "label", key),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

#[test]
fn recovered_relation_field_generation_preserves_its_complete_binding_and_entries() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let source = create_entity_outcome(execution, &runtime, "source");
    let target = create_entity_outcome(execution, &runtime, "target");
    let relation = create_relation_outcome(
        execution,
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "edge",
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(1),
            name: "relation.label.lookup".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: false,
        })
        .unwrap();
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: relation.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(built.failed_indexes.is_empty());
    let original = runtime
        .index_access()
        .read_latest_generation(execution, index.index_id, &BranchId("main".into()))
        .unwrap()
        .unwrap();
    let DerivedIndexEntries::RelationField(entries) = &original.entries else {
        panic!("relation-field generation required")
    };
    assert_eq!(
        entries
            .values()
            .flat_map(|rows| rows.iter().copied())
            .collect::<Vec<_>>(),
        changed_relations(&relation)
    );
    medium.file_checkpoint(capture(execution, &runtime));
    let (recovered, _) = recover(execution, &medium);
    let restored = recovered
        .index_access()
        .read_latest_generation(execution, index.index_id, &BranchId("main".into()))
        .unwrap()
        .unwrap();
    assert_eq!(original.as_ref(), restored.as_ref());
}

#[test]
fn recovered_entity_field_lookup_certifies_the_original_bounded_candidates() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let alpha = create_entity_outcome(execution, &runtime, "alpha");
    let field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name.lookup".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: field.clone(),
            },
            branch_scoped: false,
        })
        .unwrap();
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: alpha.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(built.failed_indexes.is_empty());
    let original = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            BoundedEntityFieldLookupRequest::new(
                alpha.snapshot.clone(),
                index.index_id,
                KindId(1),
                field.clone(),
                AspectValue::String("alpha".into()),
                16,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(original.candidate_entity_ids(), changed_entities(&alpha));
    medium.file_checkpoint(capture(execution, &runtime));
    let (recovered, _) = recover(execution, &medium);
    let snapshot = snapshot_for_branch(&recovered, &BranchId("main".into()));
    let restored = recovered
        .index_access()
        .execute_bounded_entity_field_lookup(
            BoundedEntityFieldLookupRequest::new(
                snapshot,
                index.index_id,
                KindId(1),
                field,
                AspectValue::String("alpha".into()),
                16,
            )
            .unwrap(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();
    assert_eq!(
        original.candidate_entity_ids(),
        restored.candidate_entity_ids()
    );
}
