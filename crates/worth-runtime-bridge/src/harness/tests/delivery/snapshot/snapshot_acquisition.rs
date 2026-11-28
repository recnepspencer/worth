use super::super::build_runtime;
use crate::facade::{BridgeDeliveryErrorKind, BridgeRouteRequest};
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};

#[test]
fn bridge_delivery_fails_when_newer_truth_arrives_without_required_snapshot() {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, original) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(
        truth.source.clone(),
        sink.clone(),
        vec![NamedTruth::mapping()],
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();

    let latest_identity = truth.commit_name(execution, "bob").1;
    let missing = original.snapshot_identity().clone();
    assert_ne!(latest_identity.snapshot_identity(), &missing);
    drop(original);

    let error = runtime.deliver_invalidation(route, execution).unwrap_err();
    assert!(matches!(
        error.kind(),
        BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause)
            if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::Binding(
                crate::adapter::BridgeSourceBindingDenial::SnapshotNotBound
            )
    ));
    assert_eq!(error.context().snapshot_identity(), Some(&missing));
    assert!(sink.deliveries().is_empty());
}
