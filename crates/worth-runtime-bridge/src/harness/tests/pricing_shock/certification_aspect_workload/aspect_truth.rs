use super::super::support::{
    capture_pricing_aspect_bundle, BridgeAspectRegistrationId, BridgeRuntimePolicy,
    FineGrainedMatchStatus, SubscriptionSliceKind, TruthDeltaSurfaceKind,
};
use super::EXPECTED_COST_USD_TARGET_BASIS;

#[test]
fn pricing_shock_aspect_lane_preserves_fine_grained_truth_and_history() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let aspect =
        capture_pricing_aspect_bundle(BridgeRuntimePolicy::development(), resource_request);

    assert_eq!(
        aspect.snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-aspect")
    );
    assert_eq!(
        aspect.source_branch,
        crate::truth_identity_fixtures::truth_branch_fixture("main")
    );
    assert_eq!(
        aspect.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-aspect")
    );
    assert_eq!(
        aspect.truth_surface_kind,
        TruthDeltaSurfaceKind::EntityField
    );
    assert_eq!(
        aspect.fine_grained_match_status,
        FineGrainedMatchStatus::Matched
    );
    assert_eq!(
        aspect.aspect_registration_id,
        BridgeAspectRegistrationId::admit_bridge_owned("pricing-steel-usd-field")
    );
    assert_eq!(
        aspect.subscription_slice_kind,
        SubscriptionSliceKind::SignalField
    );
    assert_eq!(
        aspect.target_canonical_basis,
        EXPECTED_COST_USD_TARGET_BASIS
    );
    assert_eq!(aspect.invalidation_target, "price:bicycle");
}
