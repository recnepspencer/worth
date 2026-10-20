//! Compare the same authoritative read with and without Bridge custody.
use super::*;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn bridge_snapshot_custody_adds_no_work_to_the_direct_native_read() {
    let runtime = runtime_with_test_schema();
    let serial = crate::snapshot::test_serial_request();
    let bootstrap = ExecutionRequest::serial(&serial);
    let created = create_entity_outcome(&bootstrap, &runtime, "visible");
    let entity = crate::relational_source::relational_test_support::changed_entities(&created)[0];
    let identity = bridge_snapshot_identity_for_handle(&created.snapshot);
    let reader = reader_for_current(runtime, identity, created.snapshot.version_id());
    let bridged = crate::facade::BridgeSnapshotContext::bind(reader.clone());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
        scalar_string_contract("name"),
    )]);
    for workers in [0, 1, 4] {
        let lease = (workers > 0).then(|| {
            crate::snapshot::test_execution_lease_for_policy(
                ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(
                        std::num::NonZeroUsize::new(workers).unwrap(),
                        None,
                        Some(100_000),
                    ),
                ),
                worth_execution::CancellationToken::new(),
            )
        });
        let request = lease.as_ref().map_or(bootstrap, ExecutionRequest::leased);
        let (direct, direct_work) = request
            .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                reader.read_packet(&packet, request).unwrap()
            })
            .unwrap();
        let (through_bridge, bridged_work) = request
            .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                bridged.read_packet(&packet, request).unwrap()
            })
            .unwrap();
        assert_eq!(
            direct.records()[0].scalar_aspect_value(),
            Some(&AspectValue::String("visible".into()))
        );
        assert_eq!(through_bridge, direct);
        assert_eq!(bridged_work.charged_work(), direct_work.charged_work());
    }
}
