use crate::facade::{
    BridgeCommittedPatchEnvelope, CommittedPatchSource, RelationalCommittedPatchRequest,
};
use crate::harness::fixtures::{ProfileTruth, RecordingSignalBridgeSink};
use crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity;

use super::support::build_runtime;

#[test]
fn bridge_artifact_identities_are_bounded_and_stable_for_identical_patchsets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let native = truth
        .native
        .source
        .load_committed_patch(
            RelationalCommittedPatchRequest::new(truth.commit.clone()),
            execution,
        )
        .unwrap();
    let item = native.patch_body().canonical_items()[0].clone();
    let envelope = |items| {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
                native.producer_metadata().clone(),
                native.commit_identity().clone(),
                native.patch_identity().clone(),
                native.snapshot_identity().clone(),
                native.branch_identity().clone(),
            ),
            items,
        )
        .expect("canonical patch envelope")
    };
    let deliver = |envelope| {
        let runtime = build_runtime(
            truth.native.source.clone(),
            RecordingSignalBridgeSink::default(),
            vec![ProfileTruth::mapping()],
        );
        let route = runtime
            .plan_envelope(envelope)
            .expect("canonical route should plan");
        runtime
            .deliver_invalidation(route, execution)
            .expect("canonical artifact should deliver")
    };
    let left_result = deliver(envelope(vec![item.clone(), item.clone()]));
    let right_result = deliver(envelope(vec![item]));

    assert_eq!(
        left_result.routing_summary().route_identity(),
        right_result.routing_summary().route_identity()
    );
    assert_eq!(
        left_result.artifact().invalidation_identity(),
        right_result.artifact().invalidation_identity()
    );
    assert_eq!(
        left_result.artifact().snapshot_token().token_value(),
        right_result.artifact().snapshot_token().token_value()
    );

    let route_identity = left_result.routing_summary().route_identity().as_str();
    let invalidation_identity = left_result.artifact().invalidation_identity().as_str();
    let snapshot_token = left_result.artifact().snapshot_token().token_value();
    assert!(route_identity.len() < 90);
    assert!(invalidation_identity.len() < 100);
    assert!(snapshot_token.len() < 100);
}
