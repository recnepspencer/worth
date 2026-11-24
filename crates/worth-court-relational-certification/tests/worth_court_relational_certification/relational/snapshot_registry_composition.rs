//! Explicit packet planning consults both genuine snapshot registries.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    query::PlannedQueryPacket,
    snapshots::SnapshotHandle,
    transactions::{CreatedEntityRef, RecordRef},
};
use worth_testkit_relational::{
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn a_known_snapshot_is_found_in_whichever_registry_holds_it() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let key = "cross-registry-witness";
    let published = create_entity_outcome(&request, &runtime, key);
    let target = RecordRef::Entity(
        published
            .created_entity(&CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: key.into(),
            })
            .unwrap(),
    );
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let active = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let absent = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let packet = |handle: &SnapshotHandle| {
        PlannedQueryPacket::explicit_targets(
            "cross-registry-packet",
            runtime.read_truth().query_plan_context(handle).unwrap(),
            vec![target.clone()],
        )
    };
    let absent_packet = packet(&absent);
    assert!(runtime
        .storage_access()
        .plan_read_explicit_query_packet(&absent, &absent_packet)
        .is_some());
    runtime.snapshots().release_snapshot(&absent).unwrap();

    // Both genuine registries keep their issued handles while the retired identity is absent.
    for handle in [&active, &published.snapshot] {
        let observed = runtime
            .storage_access()
            .plan_read_explicit_query_packet(handle, &packet(handle))
            .unwrap();
        assert_eq!(observed.target_count, 1);
    }
    assert!(runtime
        .storage_access()
        .plan_read_explicit_query_packet(&absent, &absent_packet)
        .is_none());
    runtime.snapshots().release_snapshot(&active).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&published.snapshot)
        .unwrap();
}
