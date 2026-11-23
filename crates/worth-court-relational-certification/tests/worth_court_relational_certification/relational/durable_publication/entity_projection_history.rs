//! Certify exact and historical projection semantics through published snapshots.
use super::branch_history_publications::delete_entity;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        ProjectionAspectScope, ProjectionContractRefusal, ProjectionRecordDomain,
        RelationalProjectionContractDenial,
    },
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    aspect_storage::{aspect_key, field_key, publications::changed_entities},
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn borrowed_projection_cannot_bypass_installed_mask_contract() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let created = create_entity_outcome(execution, &runtime, "masked");
    let refusal = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_record_with_projection_scope(
            execution,
            changed_entities(&created)[0],
            ProjectionAspectScope::fields(aspect_key("name"), [field_key("alias")]),
            |_| Some(()),
        )
        .expect_err("projection contract must refuse");

    assert_eq!(
        refusal,
        RelationalExecutionDenialCause::ProjectionContract(RelationalProjectionContractDenial {
            domain: ProjectionRecordDomain::Entity,
            kind_id: KindId(1),
            requirement: Some(0),
            refusal: ProjectionContractRefusal::MaskRejected,
        })
    );
}

#[test]
fn exact_snapshot_retirement_is_metadata_only_and_does_not_change_live_projection() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
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
    assert!(before
        .read_entity_retirement(execution, entity)
        .unwrap()
        .is_none());
    let retirement = after
        .read_entity_retirement(execution, entity)
        .unwrap()
        .unwrap();
    assert_eq!(retirement.entity_id(), entity);
    assert_eq!(retirement.kind_id(), KindId(1));
    assert_eq!(retirement.created_at_version(), created.version_id);
    assert_eq!(retirement.deleted_at_version(), deleted.version_id);
    assert!(after
        .entity_record_with_projection_scope(
            execution,
            entity,
            ProjectionAspectScope::empty(),
            |_| Some(())
        )
        .expect("read request remains live")
        .is_none());
    assert!(after
        .read_entity_retirement(
            execution,
            EntityId::new(
                entity.partition_id,
                entity.local_slot_value(),
                entity.generation_value() + 1
            )
        )
        .unwrap()
        .is_none());
    assert!(after
        .read_entity_retirement(
            execution,
            EntityId::new(entity.partition_id, entity.local_slot_value(), 0)
        )
        .unwrap()
        .is_none());
    assert!(runtime
        .read_truth()
        .try_project_historical_version(deleted.version_id)
        .unwrap()
        .read_entity_retirement(execution, entity)
        .unwrap()
        .is_none());
    let foreign = memory_runtime();
    assert!(foreign
        .read_truth()
        .project_snapshot(&deleted.snapshot)
        .is_none());
}
