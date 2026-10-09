use crate::facade::transactions::MutationIntent;
use crate::tests::support::*;
use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValueView};

#[test]
fn entity_patch_aspects_follow_declared_contract_targets() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("create").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("aspect-entity"),
                fields: crate::tests::support::string_aspect_field_patch([(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "before",
                )]),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let created = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let entity = changed_entities(&created)[0];
    let created_patch = &created.patch()[0];
    let created_aspect_summary = created.aspect_summary().unwrap();

    let _ = assert_patch_truth_invariants(&created);

    assert_eq!(
        created_patch.structural_change,
        RecordStructuralChange::Created
    );
    assert_eq!(
        created_patch.authoritative_changed_aspects(),
        ordered_aspect_keys([
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("name").unwrap(),
        ])
    );
    assert!(!created_patch.contains_opaque_aspect);
    assert_eq!(created_aspect_summary.changed_entity_aspect_count, 2);
    assert_eq!(created_aspect_summary.changed_relation_aspect_count, 0);

    let updated = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::string_aspect_field_patch([(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "after",
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    let updated_patch = &updated.patch()[0];
    let updated_aspect_summary = updated.aspect_summary().unwrap();

    let _ = assert_patch_truth_invariants(&updated);

    assert_eq!(
        updated_patch.structural_change,
        RecordStructuralChange::Updated
    );
    assert_eq!(
        updated_patch.authoritative_changed_aspects(),
        ordered_aspect_keys([AspectKey::new("name").unwrap()])
    );
    let updated_read = runtime
        .read_truth()
        .read_snapshot(&updated.snapshot)
        .expect("updated snapshot should read");
    let updated_record = updated_read
        .get_entity(entity)
        .expect("updated entity should read");
    let authoritative_name = updated_record
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(&AspectKey::new("name").unwrap()))
        .expect("updated name aspect state");
    assert!(matches!(
        authoritative_name.view(),
        ContractValidatedAspectValueView::Scalar(AspectValue::String(value))
            if value == &"after".into()
    ));
    assert_eq!(updated_aspect_summary.changed_entity_aspect_count, 1);

    let idempotent_declared_update = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("idempotent-declared-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "after",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    assert_eq!(
        idempotent_declared_update.patch()[0].authoritative_changed_aspects(),
        Vec::new()
    );
    assert_eq!(
        idempotent_declared_update
            .aspect_summary()
            .unwrap()
            .changed_entity_aspect_count,
        0
    );

    let deleted = delete_entity(&runtime, entity);
    let deleted_patch = &deleted.patch()[0];
    let deleted_aspect_summary = deleted.aspect_summary().unwrap();
    assert_eq!(
        deleted_patch.structural_change,
        RecordStructuralChange::Deleted
    );
    assert_eq!(
        deleted_patch.authoritative_changed_aspects(),
        ordered_aspect_keys([
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("name").unwrap(),
        ])
    );
    assert_eq!(deleted_aspect_summary.changed_entity_aspect_count, 2);
}

#[test]
fn retained_relation_patch_only_emits_declared_lifecycle_delta_when_endpoints_and_aspects_stay_same(
) {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_outcome = create_relation_outcome(&runtime, source, target, "r-audit");
    let relation_patch = &relation_outcome.patch()[0];
    let relation_aspect_summary = relation_outcome.aspect_summary().unwrap();

    let _ = assert_patch_truth_invariants(&relation_outcome);

    assert_eq!(
        relation_patch.structural_change,
        RecordStructuralChange::Created
    );
    assert_eq!(
        relation_patch.authoritative_changed_aspects(),
        ordered_aspect_keys([
            AspectKey::new("label").unwrap(),
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("source").unwrap(),
            AspectKey::new("target").unwrap(),
        ])
    );
    assert_eq!(relation_aspect_summary.changed_relation_aspect_count, 4);

    let deleted_source = delete_entity(&runtime, source);
    let retained_relation_patch = deleted_source
        .patch()
        .iter()
        .find(|record| matches!(record.target, RecordRef::Relation(_)))
        .expect("retained relation patch");
    let deleted_source_aspect_summary = deleted_source.aspect_summary().unwrap();

    let _ = assert_patch_truth_invariants(&deleted_source);

    assert_eq!(
        retained_relation_patch.structural_change,
        RecordStructuralChange::RetainedForAudit
    );
    assert_eq!(
        retained_relation_patch.authoritative_changed_aspects(),
        ordered_aspect_keys([AspectKey::new("lifecycle").unwrap()])
    );
    assert!(!retained_relation_patch.contains_opaque_aspect);
    assert_eq!(deleted_source_aspect_summary.changed_entity_aspect_count, 2);
    assert_eq!(
        deleted_source_aspect_summary.changed_relation_aspect_count,
        1
    );
}
