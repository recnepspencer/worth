use crate::facade::{BridgeDeliveryErrorKind, BridgeRouteRequest};
use crate::harness::fixtures::InMemoryRelationalBridgeSource;

use super::super::support::{
    build_runtime_with_aspects, committed_patch, field_aspect_registration, field_slice_snapshot,
    registration, RejectingSignalSink,
};

#[test]
fn bridge_sink_rejection_records_failure_diagnostics_with_slice_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime_with_aspects(
        source,
        RejectingSignalSink,
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let route = runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("route should plan before sink rejection");
    let expected_slice_identity = route
        .lowering_summary()
        .subscription_slice_identity()
        .clone();

    let error = runtime
        .deliver_invalidation(route, execution)
        .expect_err("delivery should surface the sink rejection");

    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::SignalSinkRejection(
            crate::adapter::SignalBridgeSinkErrorKind::ExternalSinkFailure
        )
    );
    let failure = runtime
        .diagnostics()
        .last_failure_record()
        .expect("sink rejection should be recorded in diagnostics");
    assert_eq!(
        failure.subscription_slice_identity().map(|id| id.as_str()),
        Some(expected_slice_identity.as_str())
    );
    assert!(failure.invalidation_identity().is_some());
}
