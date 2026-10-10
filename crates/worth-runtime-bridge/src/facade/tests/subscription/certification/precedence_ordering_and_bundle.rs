use super::*;
use crate::facade::BridgeSubscriptionSourceArtifactRole as SourceArtifactRole;

#[test]
fn certification_multi_failure_report_promotes_highest_precedence_boundary() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let report = runtime.certify_subscription_certification_multi_failure_precedence();

    assert_ne!(
        report.control_bundle_digest(),
        report.hostile_bundle_digest()
    );
    assert_eq!(
        report.primary_failure_boundary(),
        crate::facade::BridgeSubscriptionCertificationFailureBoundary::BasisDrift
    );
    assert_eq!(
        report.primary_failure_precedence_stage(),
        crate::facade::BridgeSubscriptionCertificationFailurePrecedenceStage::BasisBinding
    );
    assert!(report.basis_drift_is_primary_without_registry_drift());
    assert!(report.suppressed_checkpoint_replay_and_diagnostics());
    assert_eq!(report.suppressed_failure_boundaries().len(), 3);
    assert!(report.suppressed_failure_boundaries().contains(
        &crate::facade::BridgeSubscriptionCertificationFailureBoundary::CheckpointDivergence
    ));
    assert!(report
        .suppressed_failure_boundaries()
        .contains(&crate::facade::BridgeSubscriptionCertificationFailureBoundary::ReplayMismatch));
    assert!(report.suppressed_failure_boundaries().contains(
        &crate::facade::BridgeSubscriptionCertificationFailureBoundary::DiagnosticsInfluence
    ));
    assert!(!report
        .suppressed_failure_boundaries()
        .contains(&crate::facade::BridgeSubscriptionCertificationFailureBoundary::RegistryDrift));
    assert!(!report.suppressed_failure_boundaries().contains(
        &crate::facade::BridgeSubscriptionCertificationFailureBoundary::DeclarationEquivalenceDrift
    ));
    assert_eq!(report.counters().comparison_plan_count(), 1);
    assert_eq!(report.counters().bundle_comparison_count(), 1);
    assert_eq!(report.counters().bundle_comparison_mismatch_count(), 4);
    assert_eq!(report.counters().failure_localization_count(), 1);
    assert_eq!(report.counters().multi_failure_precedence_report_count(), 1);
    assert_eq!(report.counters().global_history_scan_count(), 0);
    assert_eq!(report.counters().global_subscription_scan_count(), 0);
    assert!(report
        .comparison_report_digest()
        .starts_with("bridge-subscription-certification-comparison-report:sha256:"));
    assert!(report
        .digest()
        .starts_with("bridge-subscription-certification-multi-failure-precedence-report:sha256:"));
}

#[test]
fn certification_ordering_hostility_preserves_canonical_bundle_meaning() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let report = runtime.certify_subscription_certification_ordering_hostility();

    assert_eq!(
        report.control_source_artifact_index_digest(),
        report.hostile_source_artifact_index_digest()
    );
    assert_eq!(
        report.control_bundle_digest(),
        report.hostile_bundle_digest()
    );
    assert!(report.canonical_source_order_preserved());
    assert!(report.semantic_digest_preserved());
    assert!(report.sealed_bundle_digest_preserved());
    assert!(report.field_order_preserved());
    assert!(report
        .comparison_report_digest()
        .starts_with("bridge-subscription-certification-comparison-report:sha256:"));
    assert_eq!(
        report.comparison_outcome(),
        crate::facade::BridgeSubscriptionCertificationComparisonOutcome::Equivalent
    );
    assert_eq!(report.counters().source_artifact_index_entry_count(), 28);
    assert_eq!(report.counters().source_artifact_index_scan_count(), 28);
    assert_eq!(report.counters().bundle_assembly_plan_count(), 2);
    assert_eq!(report.counters().bundle_cost_profile_count(), 2);
    assert_eq!(report.counters().certification_bundle_count(), 2);
    assert_eq!(report.counters().scratch_allocation_count(), 2);
    assert_eq!(report.counters().scratch_reuse_count(), 2);
    assert_eq!(report.counters().comparison_plan_count(), 1);
    assert_eq!(report.counters().bundle_comparison_count(), 1);
    assert_eq!(report.counters().bundle_comparison_mismatch_count(), 0);
    assert_eq!(report.counters().failure_localization_count(), 0);
    assert_eq!(report.counters().ordering_hostility_report_count(), 1);
    assert_eq!(report.counters().global_history_scan_count(), 0);
    assert_eq!(report.counters().global_subscription_scan_count(), 0);
    assert!(report
        .digest()
        .starts_with("bridge-subscription-certification-ordering-hostility-report:sha256:"));
}

#[test]
fn certification_bundle_assembly_consumes_plan_cost_profile_and_scratch() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let manifest = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            component_ids(),
            lane_ids(),
        )
        .expect("manifest should seal");
    let index = runtime.build_subscription_certification_source_index(vec![source_artifact(
        crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
        SourceArtifactRole::Stable,
    )]);
    let plan = runtime.plan_subscription_certification_bundle(&manifest, &index);
    let cost_profile = runtime
        .admit_subscription_certification_cost_profile(
            crate::facade::BridgeSubscriptionCertificationDensityPosture::SparseCertificationWindow,
            8,
            16,
            32,
            false,
        )
        .expect("sparse certification profile should admit");
    let scratch = runtime.prepare_subscription_certification_scratch(&cost_profile);

    let draft = runtime
        .assemble_subscription_certification_bundle(plan, cost_profile, scratch)
        .expect("admitted sparse plan should assemble");

    assert_eq!(draft.counters().bundle_assembly_plan_count(), 1);
    assert_eq!(draft.counters().bundle_cost_profile_count(), 1);
    assert_eq!(draft.counters().certification_bundle_count(), 1);
    assert_eq!(draft.counters().source_artifact_index_scan_count(), 1);
    assert_eq!(draft.counters().scratch_allocation_count(), 1);
    assert!(draft.fields().iter().any(|field| field.field_state()
        == crate::facade::BridgeSubscriptionBundleFieldState::NotExercised));
    let comparison_inputs = draft
        .fields()
        .iter()
        .find(|field| field.field_name() == "comparison_inputs")
        .expect("comparison input field should be present for phase 2 comparison");
    assert_eq!(
        comparison_inputs.field_state(),
        crate::facade::BridgeSubscriptionBundleFieldState::Present
    );
    assert_eq!(
        comparison_inputs.field_digest(),
        draft.semantic_digests().digest()
    );
    assert_eq!(draft.completeness_report().required_field_count(), 8);
    assert_eq!(draft.completeness_report().present_field_count(), 7);
    assert_eq!(draft.completeness_report().not_exercised_field_count(), 1);
    assert_eq!(
        draft
            .completeness_report()
            .rejected_before_produced_field_count(),
        0
    );
    assert!(draft
        .semantic_digests()
        .subscription_digest()
        .starts_with("bridge-subscription-certification-subscription-digest:sha256:"));
    assert_ne!(
        draft.semantic_digests().subscription_digest(),
        draft.semantic_digests().subscription_delivery_digest()
    );
    assert!(
        !draft
            .semantic_digests()
            .strategy_lowering_digest()
            .eq(draft.semantic_digests().subscription_delivery_digest()),
        "strategy lowering must not collapse onto the delivery digest surface"
    );
    assert!(draft
        .semantic_digests()
        .counter_snapshot_digest()
        .starts_with("bridge-subscription-certification-counters:sha256:"));

    let draft_subscription_digest = draft.semantic_digests().subscription_digest().to_owned();
    let draft_strategy_lowering_digest = draft
        .semantic_digests()
        .strategy_lowering_digest()
        .to_owned();
    let draft_completeness_digest = draft.completeness_report().digest().to_owned();

    let sealed = runtime.seal_subscription_certification_bundle(draft);

    assert_eq!(
        sealed.schema_version(),
        "bridge-subscription-certification-bundle-v1"
    );
    assert_eq!(sealed.digest_algorithm(), "sha256");
    assert_eq!(sealed.counters().certification_bundle_count(), 1);
    assert_eq!(sealed.completeness_report().required_field_count(), 8);
    assert_eq!(
        sealed.semantic_digests().subscription_digest(),
        draft_subscription_digest
    );
    assert_eq!(
        sealed.semantic_digests().strategy_lowering_digest(),
        draft_strategy_lowering_digest
    );
    assert_eq!(
        sealed.completeness_report().digest(),
        draft_completeness_digest
    );
}

#[test]
fn certification_bundle_assembly_rejects_budget_mismatch_before_draft_exists() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, _active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let manifest = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            component_ids(),
            lane_ids(),
        )
        .expect("manifest should seal");
    let index = runtime.build_subscription_certification_source_index(vec![
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
            SourceArtifactRole::Stable,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::AdmittedSubscription,
            SourceArtifactRole::Stable,
        ),
    ]);
    let plan = runtime.plan_subscription_certification_bundle(&manifest, &index);
    let cost_profile = runtime
        .admit_subscription_certification_cost_profile(
            crate::facade::BridgeSubscriptionCertificationDensityPosture::SparseCertificationWindow,
            1,
            16,
            32,
            false,
        )
        .expect("sparse certification profile should admit");
    let scratch = runtime.prepare_subscription_certification_scratch(&cost_profile);

    let rejection = runtime
        .assemble_subscription_certification_bundle(plan, cost_profile, scratch)
        .expect_err("assembly must reject when indexed source artifacts exceed budget");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionCertificationAssemblyRejectionKind::SourceArtifactBudgetExceeded
    );
    assert_eq!(rejection.counters().certification_bundle_count(), 1);
    assert_eq!(rejection.counters().source_artifact_index_scan_count(), 2);
}
