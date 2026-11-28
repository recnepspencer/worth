use crate::facade::{BridgeDeliveryErrorKind, BridgeRouteRequest};
use crate::harness::fixtures::NamedTruth;

use super::super::support::{build_runtime_with_aspects, RejectingSignalSink};

#[test]
fn bridge_sink_rejection_records_failure_diagnostics_with_slice_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let runtime = build_runtime_with_aspects(
        source,
        RejectingSignalSink,
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a), execution)
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
