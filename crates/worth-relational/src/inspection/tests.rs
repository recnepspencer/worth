use crate::facade::identity::KindId;
use crate::facade::inspection::{InspectionOrigin, InspectionScope};
use crate::facade::transactions::{
    EntityReference, MutationIntent, RecordRef, RelationMutationIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
use crate::tests::support::*;

#[test]
fn structural_identity_origin_tracks_non_current_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "scoped-identity");
    let version_id = runtime.current_version_id();
    let snapshot = runtime.visibility_authority().snapshot();

    let current = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("current structural identity");
    let version = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Version(version_id),
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("version structural identity");
    let snapshot = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Snapshot(snapshot),
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("snapshot structural identity");

    assert_eq!(current.origin, InspectionOrigin::CurrentTruth);
    assert_eq!(version.origin, InspectionOrigin::VisibilitySnapshot);
    assert_eq!(snapshot.origin, InspectionOrigin::VisibilitySnapshot);
}

#[test]
fn historical_neighbors_follow_scoped_relation_endpoints_after_rewire() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let original_source = create_entity(execution, &runtime, "original-source");
    let shared_target = create_entity(execution, &runtime, "shared-target");
    let replacement_source = create_entity(execution, &runtime, "replacement-source");
    let relation = create_relation(execution, &runtime, original_source, shared_target, "edge");
    let historical_version = runtime.current_version_id();
    let historical_snapshot = runtime.visibility_authority().snapshot();

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-edge").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(replacement_source),
                target: EntityReference::Existing(shared_target),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, &runtime)
        .expect("relation rewire should commit");

    let current_neighbors = runtime
        .inspect_what_happened()
        .neighbors(execution, InspectionScope::Current, original_source)
        .expect("read request remains live");
    let historical_neighbors = runtime
        .inspect_what_happened()
        .neighbors(
            execution,
            InspectionScope::Version(historical_version),
            original_source,
        )
        .expect("read request remains live");
    let snapshot_neighbors = runtime
        .inspect_what_happened()
        .neighbors(
            execution,
            InspectionScope::Snapshot(historical_snapshot),
            original_source,
        )
        .expect("read request remains live");

    assert!(current_neighbors.outgoing_relation_ids.is_empty());
    assert_eq!(historical_neighbors.outgoing_relation_ids, vec![relation]);
    assert_eq!(snapshot_neighbors.outgoing_relation_ids, vec![relation]);
}
