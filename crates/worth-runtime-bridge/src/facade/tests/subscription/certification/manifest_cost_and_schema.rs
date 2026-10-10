use super::*;
use crate::facade::BridgeSubscriptionSourceArtifactRole as SourceArtifactRole;

#[test]
fn reference_workload_manifest_is_canonicalized_before_execution() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let manifest = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            component_ids(),
            lane_ids(),
        )
        .expect("canonical manifest should seal");

    let reordered = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids().reversed(),
            component_ids().reversed(),
            lane_ids().reversed(),
        )
        .expect("reordered manifest should seal");

    assert_eq!(manifest.digest(), reordered.digest());
    assert_eq!(manifest.product_ids().len(), 128);
    assert_eq!(manifest.component_ids().len(), 5);
}

#[test]
fn reference_workload_manifest_rejects_incomplete_fixture_contract() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let rejection = runtime
        .declare_subscription_reference_workload_manifest(
            crate::facade::BridgeSubscriptionReferenceWorkloadProductIdSet::from_declared_product_labels(
                ["product-001"],
            ),
            component_ids(),
            lane_ids(),
        )
        .expect_err("manifest must require the canonical 128-product fixture");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionReferenceWorkloadManifestRejectionKind::ProductCountMismatch
    );
}

#[test]
fn reference_workload_manifest_rejects_empty_declared_workload_ids() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let empty_product_rejection = runtime
        .declare_subscription_reference_workload_manifest(
            crate::facade::BridgeSubscriptionReferenceWorkloadProductIdSet::from_declared_product_labels(
                (0..127)
                    .map(|slot| format!("product-{slot:03}"))
                    .chain(std::iter::once(String::new())),
            ),
            component_ids(),
            lane_ids(),
        )
        .expect_err("empty product IDs must not enter sealed manifest authority");
    assert_eq!(
        empty_product_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionReferenceWorkloadManifestRejectionKind::EmptyProductId
    );

    let empty_component_rejection = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            crate::facade::BridgeSubscriptionReferenceWorkloadComponentIdSet::from_declared_component_labels([
                "steel", "rubber", "copper", "glass", "labor", "",
            ]),
            lane_ids(),
        )
        .expect_err("empty component IDs must not enter sealed manifest authority");
    assert_eq!(
        empty_component_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionReferenceWorkloadManifestRejectionKind::EmptyComponentId
    );

    let empty_lane_rejection = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            component_ids(),
            crate::facade::BridgeSubscriptionReferenceWorkloadLaneIdSet::from_declared_lane_labels(
                ["authoritative-live", ""],
            ),
        )
        .expect_err("empty lane IDs must not enter sealed manifest authority");
    assert_eq!(
        empty_lane_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionReferenceWorkloadManifestRejectionKind::EmptyLaneId
    );
}

#[test]
fn certification_source_index_is_canonical_and_scan_bounded() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let inputs = vec![
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
            SourceArtifactRole::Stable,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
            SourceArtifactRole::Stable,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::AdmittedSubscription,
            SourceArtifactRole::Stable,
        ),
    ];

    let index = runtime.build_subscription_certification_source_index(inputs);

    assert_eq!(index.records().len(), 2);
    assert_eq!(index.counters().source_artifact_index_entry_count(), 2);
    assert_eq!(index.counters().source_artifact_index_scan_count(), 3);
    assert_eq!(index.counters().global_history_scan_count(), 0);
    assert_eq!(index.counters().global_subscription_scan_count(), 0);
}

#[test]
fn certification_cost_profile_rejects_over_budget_before_assembly() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .admit_subscription_certification_cost_profile(
            crate::facade::BridgeSubscriptionCertificationDensityPosture::RejectedOverBudgetCertification,
            4,
            8,
            8,
            false,
        )
        .expect_err("over-budget certification posture should reject before assembly");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionCertificationCostProfileRejectionKind::OverBudgetPostureRejected
    );
    assert_eq!(rejection.counters().over_budget_rejection_count(), 1);
}

#[test]
fn certification_cost_profile_empty_budget_rejection_does_not_claim_over_budget_work() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .admit_subscription_certification_cost_profile(
            crate::facade::BridgeSubscriptionCertificationDensityPosture::SparseCertificationWindow,
            0,
            8,
            8,
            false,
        )
        .expect_err("empty source artifact budgets must reject before posture work");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionCertificationCostProfileRejectionKind::EmptySourceArtifactBudget
    );
    assert_eq!(rejection.counters().over_budget_rejection_count(), 0);
}

#[test]
fn certification_cost_posture_report_proves_dense_over_budget_and_scratch_lifecycle() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let report = runtime.certify_subscription_certification_cost_posture();

    assert!(report.dense_selected_before_assembly());
    assert!(report.over_budget_rejected_before_assembly());
    assert!(report.scratch_lifecycle_reuse_visible());
    assert!(report
        .sparse_cost_profile_digest()
        .starts_with("bridge-subscription-certification-cost-profile:sha256:"));
    assert!(report
        .dense_cost_profile_digest()
        .starts_with("bridge-subscription-certification-cost-profile:sha256:"));
    assert_ne!(
        report.sparse_cost_profile_digest(),
        report.dense_cost_profile_digest()
    );
    assert!(report
        .over_budget_rejection_digest()
        .starts_with("bridge-subscription-certification-cost-profile-rejection:sha256:"));
    assert_eq!(
        report.first_scratch_digest(),
        report.repeated_scratch_digest()
    );
    assert_eq!(report.counters().bundle_cost_profile_count(), 2);
    assert_eq!(report.counters().dense_rebuild_count(), 1);
    assert_eq!(report.counters().over_budget_rejection_count(), 1);
    assert_eq!(report.counters().scratch_allocation_count(), 1);
    assert_eq!(report.counters().scratch_reuse_count(), 2);
    assert_eq!(report.counters().cost_posture_report_count(), 1);
    assert_eq!(report.counters().certification_bundle_count(), 0);
    assert_eq!(report.counters().global_history_scan_count(), 0);
    assert_eq!(report.counters().global_subscription_scan_count(), 0);
    assert!(report
        .digest()
        .starts_with("bridge-subscription-certification-cost-posture-report:sha256:"));
}

#[test]
fn certification_schema_parity_report_preempts_semantic_drift() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let report = runtime.certify_subscription_certification_schema_parity();

    assert_ne!(
        report.parity_bundle_digest(),
        report.divergent_bundle_digest()
    );
    assert_eq!(
        report.parity_schema_version(),
        "bridge-subscription-certification-bundle-v1"
    );
    assert_eq!(
        report.divergent_schema_version(),
        "bridge-subscription-certification-bundle-v999"
    );
    assert_eq!(report.parity_digest_algorithm(), "sha256");
    assert_eq!(report.divergent_digest_algorithm(), "sha512");
    assert_eq!(
        report.primary_failure_boundary(),
        crate::facade::BridgeSubscriptionCertificationFailureBoundary::BundleSchemaOrDigestDivergence
    );
    assert_eq!(
        report.primary_failure_precedence_stage(),
        crate::facade::BridgeSubscriptionCertificationFailurePrecedenceStage::BundleSchemaParity
    );
    assert_eq!(report.suppressed_failure_boundary_count(), 0);
    assert!(report.semantic_drift_shadowed_by_schema_divergence());
    assert_eq!(report.counters().comparison_plan_count(), 1);
    assert_eq!(report.counters().bundle_comparison_count(), 1);
    assert_eq!(report.counters().bundle_comparison_mismatch_count(), 1);
    assert_eq!(report.counters().failure_localization_count(), 1);
    assert_eq!(report.counters().schema_parity_report_count(), 1);
    assert_eq!(report.counters().global_history_scan_count(), 0);
    assert_eq!(report.counters().global_subscription_scan_count(), 0);
    assert!(report
        .comparison_report_digest()
        .starts_with("bridge-subscription-certification-comparison-report:sha256:"));
    assert!(report
        .digest()
        .starts_with("bridge-subscription-certification-schema-parity-report:sha256:"));
}
