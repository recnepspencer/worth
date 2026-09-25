use super::*;
use crate::facade::{
    config::CascadeDeletePolicy,
    identity::{KindId, PartitionId},
    mvcc::WorkerIntentBatch,
    transactions::{CreateIntent, MutationIntent},
};
use crate::storage::data::RecordLifecycleState;
use crate::tests::support::*;
use worth_foundational::facade::ContractValidatedAspectValueView;

#[test]
fn scoped_field_projection_borrows_selected_root_without_copying_unrelated_payload() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("projection").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("projection"),
                    fields: string_aspect_field_patch([
                        (aspect_key("name"), field_key("name"), "before"),
                        (aspect_key("summary"), field_key("title"), "selected"),
                        (
                            aspect_key("summary"),
                            field_key("status"),
                            &"x".repeat(256 * 1024),
                        ),
                    ]),
                }),
            )),
        )
        .unwrap();
    let created = transaction.commit(&runtime).unwrap();
    let entity = changed_entities(&created)[0];
    let updated = update_entity(&runtime, entity, "after");
    for (snapshot, expected_name) in [(&created.snapshot, "before"), (&updated.snapshot, "after")] {
        let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
        assert_borrowed_title(&view, entity);
        let name = view.entity_record_with_projection_scope(
            entity,
            ProjectionAspectScope::whole_aspects([aspect_key("name")]),
            |record| {
                assert_eq!(record.entity_id(), entity);
                assert_eq!(record.kind_id(), KindId(1));
                assert_eq!(record.lifecycle(), RecordLifecycleState::Live);
                assert_eq!(record.created_at_version(), created.version_id);
                record.aspect_value(&aspect_key("name")).cloned()
            },
        );
        assert_eq!(name, Some(string_aspect_value(expected_name)));
    }
}

fn assert_borrowed_title(view: &VisibilityProjectionView<'_>, entity: EntityId) {
    let root = view.basis.root().unwrap();
    let partition = root.get_partition(entity.partition_id).unwrap();
    let slot = partition
        .entity_arena
        .get_slot(entity.slot_index())
        .unwrap();
    let state = slot.extra().authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Struct(summary) =
        state.get(&aspect_key("summary")).unwrap().view()
    else {
        panic!("struct fixture")
    };
    let title = summary.get(&field_key("title")).unwrap();
    let seen = view.entity_record_with_projection_scope(
        entity,
        ProjectionAspectScope::fields(aspect_key("summary"), [field_key("title")]),
        |record| {
            let projected =
                record.aspect_field_value(&aspect_key("summary"), &field_key("title"))?;
            // The public callback must borrow the native value, not a materialized clone.
            assert!(std::ptr::eq(projected, title));
            assert!(record
                .aspect_field_value(&aspect_key("summary"), &field_key("status"))
                .is_none());
            assert!(record.struct_aspect_value(&aspect_key("summary")).is_none());
            Some(())
        },
    );
    assert_eq!(seen, Some(()));
}

#[test]
fn projection_preserves_identity_kind_and_lifecycle_across_exact_and_historical_bases() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "live");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(&runtime, entity);
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let after = runtime
        .read_truth()
        .project_snapshot(&deleted.snapshot)
        .unwrap();
    let historical_before = runtime
        .read_truth()
        .project_historical_version(created.version_id);
    let historical_after = runtime
        .read_truth()
        .project_historical_version(deleted.version_id);
    for (before, after) in [(before, after), (historical_before, historical_after)] {
        assert_projection_lifecycle(&before, &after, entity);
    }
}

fn assert_projection_lifecycle(
    before: &VisibilityProjectionView<'_>,
    after: &VisibilityProjectionView<'_>,
    entity: EntityId,
) {
    let scope = ProjectionAspectScope::whole_aspects([aspect_key("name")]);
    let value =
        |record: EntityProjectionRecord<'_>| record.aspect_value(&aspect_key("name")).cloned();
    assert_eq!(
        before.entity_record_with_projection_scope(entity, scope.clone(), value),
        Some(string_aspect_value("live"))
    );
    assert!(after
        .entity_record_with_projection_scope(entity, scope.clone(), value)
        .is_none());
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value() + 1,
    );
    assert!(before
        .entity_record_with_projection_scope(stale, scope.clone(), value)
        .is_none());
    assert_eq!(
        before.entity_record_of_expected_kind_with_projection_scope(
            entity,
            KindId(2),
            scope.clone(),
            value
        ),
        Err(KindId(1))
    );
    let wildcard = EntityId::new(entity.partition_id, entity.local_slot_value(), 0);
    assert_eq!(
        before.entity_record_with_projection_scope(wildcard, scope, |record| Some(
            record.entity_id()
        )),
        Some(entity)
    );
}

#[test]
#[should_panic(expected = "projection mask rejected by aspect contract")]
fn borrowed_projection_cannot_bypass_installed_mask_contract() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "masked");
    runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_record_with_projection_scope(
            changed_entities(&created)[0],
            ProjectionAspectScope::fields(aspect_key("name"), [field_key("name")]),
            |_| Some(()),
        );
}
