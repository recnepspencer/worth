use super::*;
use worth_execution::{
    CancellationToken, ExecutionMemoryReservation, ExecutionRequest, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn snapshot_serial_memory_is_refused_after_scope_admission() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "visible");
    let identity = bridge_snapshot_identity_for_handle(&created.snapshot);
    let reader = reader_for_current(runtime, identity, created.snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(1, 999, 1),
        scalar_string_contract("name"),
    )]);
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 4096, 1000),
    );
    let serial = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let error = request
        .in_scope(|lease| {
            // A caller retains all available bytes after admitting its scope.
            // The refusal's offered remainder is public budget information.
            let worth_execution::LeaseDenial::MemoryExhausted(refusal) =
                ExecutionMemoryReservation::reserve_in_scope(
                    lease,
                    policy.budget().charged_memory_bytes(),
                )
                .unwrap_err()
            else {
                panic!("scope metadata consumes some memory");
            };
            let held =
                ExecutionMemoryReservation::reserve_in_scope(lease, refusal.admitted).unwrap();
            // Read in the same scope so the denial is the records' reservation,
            // rather than a second scope's metadata admission.
            let denied = super::super::snapshot_reading::read_packet_for_serial_test(
                &reader, &packet, lease, request,
            );
            drop(held);
            denied.unwrap_err()
        })
        .unwrap();
    let crate::facade::BridgeSnapshotReadErrorKind::ExecutionDenied(
        crate::facade::BridgeExecutionDenial::MemoryExhausted(memory),
    ) = error.kind()
    else {
        panic!("serial allocation must preserve its memory cause: {error:?}");
    };
    assert_eq!(memory.admitted, 0);
    assert_eq!(
        memory.requested,
        std::mem::size_of::<crate::facade::SnapshotReadRecord>() as u64
    );
}

#[test]
fn snapshot_result_clone_shares_reserved_record_storage() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "visible");
    let identity = bridge_snapshot_identity_for_handle(&created.snapshot);
    let reader = reader_for_current(runtime, identity, created.snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(1, 999, 1),
        scalar_string_contract("name"),
    )]);
    let serial = crate::snapshot::test_serial_request();
    let result = reader
        .read_packet(&packet, ExecutionRequest::serial(&serial))
        .unwrap();
    let cloned = result.clone();
    assert_eq!(result.records().len(), 1);
    assert_eq!(result.records().as_ptr(), cloned.records().as_ptr());
    let validated =
        crate::snapshot::validate_snapshot_read_result_contract(&packet, result).unwrap();
    let validated_clone = validated.clone();
    assert_eq!(
        validated.records().as_ptr(),
        validated_clone.records().as_ptr()
    );
    drop(cloned);
    drop(validated);
    assert_eq!(validated_clone.records().len(), 1);
}
