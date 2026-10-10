use super::super::support::*;
use crate::facade::{
    BridgeTemporalAsyncCertificationBundleComparisonOutcome,
    BridgeTemporalAsyncCertificationDiagnosticsRichness,
};

#[test]
fn equivalent_temporal_async_subscription_bundles_compare_equal() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let left_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let right_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let left = left_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &left_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));
    let right =
        right_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &right_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            "temporal-commit-a",
            "temporal-snapshot-a",
            resource_request,
        ));

    let comparison = left_runtime.compare_temporal_async_certification_bundles(&left, &right);

    assert_eq!(
        comparison.outcome(),
        BridgeTemporalAsyncCertificationBundleComparisonOutcome::Equivalent
    );
    assert!(!comparison.diagnostics_richness_only_delta());
    assert!(comparison.mismatched_sections().is_empty());
}

#[test]
fn diagnostics_richness_does_not_change_semantic_bundle_parity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let minimal_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let rich_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let minimal =
        minimal_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &minimal_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            "temporal-commit-a",
            "temporal-snapshot-a",
            resource_request,
        ));
    let rich = rich_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &rich_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Rich,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));

    let comparison = minimal_runtime.compare_temporal_async_certification_bundles(&minimal, &rich);

    assert_eq!(
        comparison.outcome(),
        BridgeTemporalAsyncCertificationBundleComparisonOutcome::DiagnosticsRichnessOnlyDelta
    );
    assert!(comparison.diagnostics_richness_only_delta());
}

#[test]
fn temporal_basis_change_breaks_bundle_parity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let left_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let right_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let left = left_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &left_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));
    let right =
        right_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &right_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            "temporal-commit-b",
            "temporal-snapshot-b",
            resource_request,
        ));

    let comparison = left_runtime.compare_temporal_async_certification_bundles(&left, &right);

    assert_eq!(
        comparison.outcome(),
        BridgeTemporalAsyncCertificationBundleComparisonOutcome::Divergent
    );
    assert!(comparison
        .mismatched_sections()
        .iter()
        .any(|section| matches!(
            section,
            crate::facade::BridgeTemporalAsyncCertificationBundleMismatchSection::TemporalBasis
        )));
}

#[test]
fn inspection_and_export_preserve_section_traceability() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = crate::facade::tests::source::support::runtime_with_authority();
    let bundle = runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));

    let inspection = runtime.inspect_temporal_async_certification_bundle(&bundle);
    let export = runtime.export_temporal_async_certification_bundle(&bundle);

    assert_eq!(bundle.basis_section().truth_owner(), "worth-relational");
    assert_eq!(bundle.basis_section().signal_owner(), "worth-signal");
    assert_eq!(
        bundle.async_section().bridge_owner(),
        "worth-runtime-bridge"
    );
    assert_eq!(
        bundle.mixed_cause_section().bridge_owner(),
        "worth-runtime-bridge"
    );
    assert_eq!(
        bundle.resume_section().bridge_owner(),
        "worth-runtime-bridge"
    );
    assert_eq!(
        bundle.failure_section().bridge_owner(),
        "worth-runtime-bridge"
    );
    assert_eq!(inspection.bundle_digest(), bundle.digest());
    assert_eq!(export.bundle_digest(), bundle.digest());
    assert!(export.export_name().contains(bundle.schema_version()));
}

#[test]
fn shared_delivery_consumer_identity_change_breaks_bundle_parity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let left_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let right_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let left = left_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &left_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));
    let ready = activation_ready_detail_subscription_in_runtime(&right_runtime, resource_request);
    let cost_profile = right_runtime
        .admit_subscription_delivery_cost_profile(
            crate::facade::BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            2,
        )
        .expect("cost profile should admit");
    let alternate_consumer = right_runtime
        .admit_subscription_consumer_contract(
            crate::facade::BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            crate::facade::BridgeSubscriptionConsumerPacingCapability::Immediate,
            crate::facade::BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            false,
            crate::facade::BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("alternate consumer contract should admit");
    let active = right_runtime.activate_subscription_delivery(
        ready,
        cost_profile,
        alternate_consumer.clone(),
    );
    let alternate_bundle = shared_delivery_bundle_for_consumers(
        &right_runtime,
        &active,
        crate::facade::BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        vec![alternate_consumer],
    );
    let right = right_runtime.seal_temporal_async_certification_bundle(
        temporal_async_bundle_draft_with_shared_bundle(
            &right_runtime,
            active,
            alternate_bundle,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            "temporal-commit-a",
            "temporal-snapshot-a",
        ),
    );

    let comparison = left_runtime.compare_temporal_async_certification_bundles(&left, &right);

    assert_eq!(
        comparison.outcome(),
        BridgeTemporalAsyncCertificationBundleComparisonOutcome::Divergent
    );
    assert!(comparison
        .mismatched_sections()
        .iter()
        .any(|section| matches!(
        section,
        crate::facade::BridgeTemporalAsyncCertificationBundleMismatchSection::MixedCauseDelivery
    )));
}

#[test]
fn diagnostics_richness_changes_export_name_without_changing_semantic_parity() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let minimal_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let rich_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let minimal =
        minimal_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &minimal_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            "temporal-commit-a",
            "temporal-snapshot-a",
            resource_request,
        ));
    let rich = rich_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &rich_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Rich,
        "temporal-commit-a",
        "temporal-snapshot-a",
        resource_request,
    ));

    let minimal_export = minimal_runtime.export_temporal_async_certification_bundle(&minimal);
    let rich_export = rich_runtime.export_temporal_async_certification_bundle(&rich);
    let comparison = minimal_runtime.compare_temporal_async_certification_bundles(&minimal, &rich);

    assert_ne!(minimal_export.export_name(), rich_export.export_name());
    assert_eq!(
        comparison.outcome(),
        BridgeTemporalAsyncCertificationBundleComparisonOutcome::DiagnosticsRichnessOnlyDelta
    );
}
