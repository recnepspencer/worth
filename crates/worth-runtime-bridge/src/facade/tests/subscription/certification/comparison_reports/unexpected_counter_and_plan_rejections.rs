use super::*;
use crate::facade::BridgeSubscriptionSourceArtifactRole as SourceArtifactRole;

#[test]
fn certification_comparison_reports_unexpected_rejection_boundary() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let mut left_inputs =
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    left_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::BasisBinding,
        SourceArtifactRole::Left,
    ));
    let left = sealed_certification_bundle(&runtime, left_inputs, false);
    let mut right_inputs =
        active_source_inputs(SourceArtifactRole::Divergent, SourceArtifactRole::Stable);
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::BasisBinding,
        SourceArtifactRole::Right,
    ));
    let right = sealed_certification_bundle(&runtime, right_inputs, false);
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ExpectedRejection,
            Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::BasisDrift),
            None,
        )
        .expect("expected rejection plan should admit with boundary");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::RejectedAtUnexpectedBoundary
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::RegistryDrift)
    );
    assert!(report
        .suppressed_failure_boundaries()
        .contains(&crate::facade::BridgeSubscriptionCertificationFailureBoundary::BasisDrift));
}

#[test]
fn certification_comparison_detects_counter_contract_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let inputs = active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    let mut duplicate_scan_inputs = inputs.clone();
    duplicate_scan_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
        SourceArtifactRole::Stable,
    ));
    let left = sealed_certification_bundle(&runtime, inputs, false);
    let right = sealed_certification_bundle(&runtime, duplicate_scan_inputs, false);
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::CounterContract,
            None,
            None,
        )
        .expect("counter contract plan should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        left.semantic_digests().subscription_digest(),
        right.semantic_digests().subscription_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_registry_digest(),
        right.semantic_digests().subscription_registry_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_basis_digest(),
        right.semantic_digests().subscription_basis_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_lifecycle_digest(),
        right.semantic_digests().subscription_lifecycle_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_delivery_digest(),
        right.semantic_digests().subscription_delivery_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_share_digest(),
        right.semantic_digests().subscription_share_digest()
    );
    assert_eq!(
        left.semantic_digests().subscription_continuation_digest(),
        right.semantic_digests().subscription_continuation_digest()
    );
    assert_eq!(
        left.semantic_digests().consumer_contract_digest(),
        right.semantic_digests().consumer_contract_digest()
    );
    assert_eq!(
        left.semantic_digests().checkpoint_digest(),
        right.semantic_digests().checkpoint_digest()
    );
    assert_eq!(
        left.semantic_digests().routing_digest(),
        right.semantic_digests().routing_digest()
    );
    assert_eq!(
        left.semantic_digests().replay_digest(),
        right.semantic_digests().replay_digest()
    );
    assert_eq!(
        left.semantic_digests().diagnostics_digest(),
        right.semantic_digests().diagnostics_digest()
    );
    assert_eq!(
        left.semantic_digests().failure_digest(),
        right.semantic_digests().failure_digest()
    );
    assert_eq!(
        left.semantic_digests().residue_digest(),
        right.semantic_digests().residue_digest()
    );
    assert_eq!(
        left.semantic_digests().strategy_lowering_digest(),
        right.semantic_digests().strategy_lowering_digest()
    );
    assert_ne!(
        left.semantic_digests().counter_snapshot_digest(),
        right.semantic_digests().counter_snapshot_digest()
    );
    assert_ne!(left.counters().digest(), right.counters().digest());
    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::CounterContractViolation
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(
            crate::facade::BridgeSubscriptionCertificationFailureBoundary::CounterContractViolation
        )
    );
    assert_eq!(report.mismatch_count(), 1);
}

#[test]
fn certification_comparison_plan_rejects_expected_rejection_without_boundary() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ExpectedRejection,
            None,
            None,
        )
        .expect_err("expected rejection plans must name a boundary");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionCertificationComparisonPlanRejectionKind::ExpectedRejectionRequiresBoundary
    );
}

#[test]
fn certification_comparison_plan_rejects_intentional_divergence_without_axis() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::IntentionalDivergence,
            None,
            None,
        )
        .expect_err("intentional divergence plans must name an axis");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionCertificationComparisonPlanRejectionKind::IntentionalDivergenceRequiresAxis
    );
}
