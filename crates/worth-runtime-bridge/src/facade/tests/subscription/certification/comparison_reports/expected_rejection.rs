use super::*;

#[test]
fn certification_comparison_reports_expected_rejection_with_precedence() {
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
        active_source_inputs(SourceArtifactRole::Divergent, SourceArtifactRole::Divergent);
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::BasisBinding,
        SourceArtifactRole::Right,
    ));
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
        SourceArtifactRole::Right,
    ));
    let right = sealed_certification_bundle(&runtime, right_inputs, true);
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ExpectedRejection,
            Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::RegistryDrift),
            None,
        )
        .expect("expected rejection plan should admit with boundary");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::RejectedAtExpectedBoundary
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::RegistryDrift)
    );
    assert_eq!(
        report.primary_failure_precedence_stage(),
        Some(
            crate::facade::BridgeSubscriptionCertificationFailurePrecedenceStage::DeclarationOrRegistry
        )
    );
    assert!(report
        .suppressed_failure_boundaries()
        .contains(&crate::facade::BridgeSubscriptionCertificationFailureBoundary::BasisDrift));
    assert!(report.suppressed_failure_boundaries().contains(
        &crate::facade::BridgeSubscriptionCertificationFailureBoundary::DiagnosticsInfluence
    ));
    assert!(report.counters().failure_localization_count() > 0);
}
