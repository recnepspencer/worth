use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::KindId;
use crate::tests::support::*;

#[test]
fn projection_preserves_identity_kind_and_lifecycle_across_exact_and_historical_bases() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "live");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(execution, &runtime, entity);
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
        .try_project_historical_version(created.version_id)
        .unwrap();
    let historical_after = runtime
        .read_truth()
        .try_project_historical_version(deleted.version_id)
        .unwrap();
    for (before, after) in [(before, after), (historical_before, historical_after)] {
        assert_projection_lifecycle(execution, &before, &after, entity);
    }
}

fn assert_projection_lifecycle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &VisibilityProjectionView,
    after: &VisibilityProjectionView,
    entity: EntityId,
) {
    let scope = ProjectionAspectScope::whole_aspects([aspect_key("name")]);
    let value =
        |record: EntityProjectionRecord<'_>| record.aspect_value(&aspect_key("name")).cloned();
    assert_eq!(
        before
            .entity_record_with_projection_scope(execution, entity, scope.clone(), value)
            .expect("read request remains live"),
        Some(string_aspect_value("live"))
    );
    assert!(after
        .entity_record_with_projection_scope(execution, entity, scope.clone(), value)
        .expect("read request remains live")
        .is_none());
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value() + 1,
    );
    assert!(before
        .entity_record_with_projection_scope(execution, stale, scope.clone(), value)
        .expect("read request remains live")
        .is_none());
    assert_eq!(
        before
            .entity_record_of_expected_kind_with_projection_scope(
                execution,
                entity,
                KindId(2),
                scope.clone(),
                value
            )
            .expect("read request remains live"),
        Err(KindId(1))
    );
    let wildcard = EntityId::new(entity.partition_id, entity.local_slot_value(), 0);
    assert_eq!(
        before
            .entity_record_with_projection_scope(execution, wildcard, scope, |record| Some(
                record.entity_id()
            ))
            .expect("read request remains live"),
        Some(entity)
    );
}

#[test]
fn wrong_kind_refusal_precedes_projection_mask_admission() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "wrong-kind");
    let entity = changed_entities(&created)[0];
    for view in [
        runtime
            .read_truth()
            .project_snapshot(&created.snapshot)
            .unwrap(),
        runtime
            .read_truth()
            .try_project_historical_version(created.version_id)
            .unwrap(),
    ] {
        let called = std::cell::Cell::new(false);
        let result = view.entity_record_of_expected_kind_with_projection_scope(
            execution,
            entity,
            KindId(2),
            ProjectionAspectScope::fields(aspect_key("undeclared"), [field_key("field")]),
            |_| {
                called.set(true);
                Some(())
            },
        );
        assert_eq!(result, Ok(Err(KindId(1))));
        assert!(
            !called.get(),
            "a wrong-kind record must not reach the projection callback"
        );
    }
}
