mod expected_rejection;

mod unexpected_counter_and_plan_rejections;

use super::*;
use crate::facade::BridgeSubscriptionSourceArtifactRole as SourceArtifactRole;

#[test]
fn certification_comparison_reports_semantic_equivalence_from_sealed_bundles() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable),
        false,
    );
    let right = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable),
        false,
    );
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::SemanticEquivalence,
            None,
            None,
        )
        .expect("semantic equivalence plan should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(left.digest(), right.digest());
    assert_eq!(report.left_bundle_digest(), left.digest());
    assert_eq!(report.right_bundle_digest(), right.digest());
    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::Equivalent
    );
    assert_eq!(report.primary_failure_boundary(), None);
    assert_eq!(report.counters().comparison_plan_count(), 1);
    assert_eq!(report.counters().bundle_comparison_count(), 1);
    assert_eq!(report.counters().bundle_comparison_mismatch_count(), 0);
}

#[test]
fn certification_comparison_reports_diagnostics_only_variation() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable),
        false,
    );
    let right = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable),
        true,
    );
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::DiagnosticsOnlyVariation,
            None,
            None,
        )
        .expect("diagnostics variation plan should admit");

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
        left.semantic_digests().diagnostics_digest(),
        right.semantic_digests().diagnostics_digest()
    );
    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::DiagnosticsOnlyDifference
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::DiagnosticsInfluence)
    );
    assert_eq!(report.mismatch_count(), 1);
    assert!(report.suppressed_failure_boundaries().is_empty());
}

#[test]
fn certification_comparison_reports_intentional_strategy_divergence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable),
        false,
    );
    let right = sealed_certification_bundle(
        &runtime,
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Divergent),
        false,
    );
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::IntentionalDivergence,
            None,
            Some(crate::facade::BridgeSubscriptionCertificationDivergenceAxis::StrategyLowering),
        )
        .expect("intentional strategy divergence plan should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::IntentionallyDivergent
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(
            crate::facade::BridgeSubscriptionCertificationFailureBoundary::StrategyLoweringProvenanceMismatch
        )
    );
}

#[test]
fn certification_comparison_reports_replay_mismatch_distinctly() {
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
        crate::facade::BridgeSubscriptionSourceArtifactKind::RetainedReplay,
        SourceArtifactRole::Left,
    ));
    let mut right_inputs =
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::RetainedReplay,
        SourceArtifactRole::Right,
    ));
    let left = sealed_certification_bundle(&runtime, left_inputs, false);
    let right = sealed_certification_bundle(&runtime, right_inputs, false);
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ReplayEquivalence,
            None,
            None,
        )
        .expect("replay equivalence plan should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::ReplayMismatch
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::ReplayMismatch)
    );
    assert_eq!(report.mismatch_count(), 1);
}

#[test]
fn certification_comparison_reports_residue_mismatch_distinctly() {
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
        crate::facade::BridgeSubscriptionSourceArtifactKind::Preview,
        SourceArtifactRole::Left,
    ));
    let mut right_inputs =
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::Preview,
        SourceArtifactRole::Right,
    ));
    let left = sealed_certification_bundle(&runtime, left_inputs, false);
    let right = sealed_certification_bundle(&runtime, right_inputs, false);
    for bundle in [&left, &right] {
        let preview_records = bundle
            .fields()
            .iter()
            .find(|field| field.field_name() == "preview_records")
            .expect("preview residue source must produce an inspectable preview field");
        assert_eq!(
            preview_records.field_state(),
            crate::facade::BridgeSubscriptionBundleFieldState::Present
        );
        assert_eq!(
            preview_records.field_digest(),
            bundle.semantic_digests().residue_digest()
        );
    }
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ResidueAbsence,
            None,
            None,
        )
        .expect("residue absence plan should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::ResidueMismatch
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::PreviewResidueMismatch)
    );
    assert_eq!(report.mismatch_count(), 1);
}

#[test]
fn certification_comparison_localizes_failure_record_drift_before_counters() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left_inputs = active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    let mut right_inputs =
        active_source_inputs(SourceArtifactRole::Stable, SourceArtifactRole::Stable);
    right_inputs.push(source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::Failure,
        SourceArtifactRole::Hostile,
    ));
    let left = sealed_certification_bundle(&runtime, left_inputs, false);
    let right = sealed_certification_bundle(&runtime, right_inputs, false);
    let plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ExpectedRejection,
            Some(
                crate::facade::BridgeSubscriptionCertificationFailureBoundary::MissingRequiredRetainedArtifact,
            ),
            None,
        )
        .expect("expected missing artifact comparison should admit");

    let report = runtime.compare_subscription_certification_bundles(plan, &left, &right);

    assert_eq!(
        report.outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::RejectedAtExpectedBoundary
    );
    assert_eq!(
        report.primary_failure_boundary(),
        Some(
            crate::facade::BridgeSubscriptionCertificationFailureBoundary::MissingRequiredRetainedArtifact
        )
    );
    assert!(report.suppressed_failure_boundaries().contains(
        &crate::facade::BridgeSubscriptionCertificationFailureBoundary::CounterContractViolation
    ));
}
