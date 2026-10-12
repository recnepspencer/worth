use super::*;
use crate::facade::config::CascadeDeletePolicy;
use crate::tests::support::*;

#[test]
fn exact_snapshot_retirement_is_metadata_only_and_does_not_change_live_projection() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "live");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(execution, &runtime, entity);
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot, execution)
        .unwrap();
    let after = runtime
        .read_truth()
        .project_snapshot(&deleted.snapshot, execution)
        .unwrap();
    assert!(before.entity_retirement(entity).is_none());
    let retirement = after.entity_retirement(entity).unwrap();
    assert_eq!(retirement.entity_id(), entity);
    assert_eq!(retirement.kind_id(), KindId(1));
    assert_eq!(retirement.created_at_version(), created.version_id);
    assert_eq!(retirement.deleted_at_version(), deleted.version_id);
    assert!(after
        .entity_record_with_projection_scope(
            execution,
            entity,
            super::super::ProjectionAspectScope::empty(),
            |_| Some(())
        )
        .expect("read request remains live")
        .is_none());
    assert!(after
        .entity_retirement(EntityId::new(
            entity.partition_id,
            entity.local_slot_value(),
            entity.generation_value() + 1
        ))
        .is_none());
    assert!(after
        .entity_retirement(EntityId::new(
            entity.partition_id,
            entity.local_slot_value(),
            0
        ))
        .is_none());
    assert!(runtime
        .read_truth()
        .project_historical_version(deleted.version_id)
        .entity_retirement(entity)
        .is_none());
    let foreign = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    assert!(foreign
        .read_truth()
        .project_snapshot(&deleted.snapshot, execution)
        .is_none());
}
