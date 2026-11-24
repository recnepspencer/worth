//! Exact projection preserves native value custody and selected lifecycle.
use super::proposal_records::update_name;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::{ProjectionAspectScope, VisibilityProjectionView},
    storage::RecordLifecycleState,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, entity_field_aspect, entity_summary_struct_aspect, field_key,
        publications::{
            begin_main_transaction, changed_entities, string_aspect_field_patch,
            string_aspect_value,
        },
        AspectSchemaFixture,
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn scoped_field_projection_borrows_selected_root_without_copying_unrelated_payload() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("projection").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "projection".into(),
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let created = transaction.commit(execution, &runtime).unwrap();
    let entity = changed_entities(&created)[0];
    let updated = update_name(execution, &runtime, entity, "after");
    for (snapshot, expected_name) in [(&created.snapshot, "before"), (&updated.snapshot, "after")] {
        let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
        assert_borrowed_title(execution, &view, entity);
        let name = view
            .entity_record_with_projection_scope(
                execution,
                entity,
                ProjectionAspectScope::whole_aspects([aspect_key("name")]),
                |record| {
                    assert_eq!(record.entity_id(), entity);
                    assert_eq!(record.kind_id(), KindId(1));
                    assert_eq!(record.lifecycle(), RecordLifecycleState::Live);
                    assert_eq!(record.created_at_version(), created.version_id);
                    record.aspect_value(&aspect_key("name")).cloned()
                },
            )
            .expect("read request remains live");
        assert_eq!(name, Some(string_aspect_value(expected_name)));
    }
}

fn assert_borrowed_title(
    execution: &ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    entity: EntityId,
) {
    view.read_exact_entity_state(execution, entity, |_, state| {
        let state = state.expect("native entity has authoritative aspects");
        let ContractValidatedAspectValueView::Struct(summary) =
            state.get(&aspect_key("summary")).unwrap().view()
        else {
            panic!("struct fixture")
        };
        let title = summary.get(&field_key("title")).unwrap();
        let seen = view
            .entity_record_with_projection_scope(
                execution,
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
            )
            .expect("read request remains live");
        assert_eq!(seen, Some(()));
    })
    .unwrap()
    .expect("exact entity remains visible");
}
