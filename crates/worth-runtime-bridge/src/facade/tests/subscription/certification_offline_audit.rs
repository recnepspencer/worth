use super::support::*;
use crate::facade::{
    BridgeSubscriptionReferenceWorkloadComponentIdSet,
    BridgeSubscriptionReferenceWorkloadLaneIdSet, BridgeSubscriptionReferenceWorkloadProductIdSet,
    BridgeSubscriptionSourceArtifactRole as SourceArtifactRole, RuntimeBridge,
};

fn product_ids() -> BridgeSubscriptionReferenceWorkloadProductIdSet {
    BridgeSubscriptionReferenceWorkloadProductIdSet::from_declared_product_labels(
        (0..128).map(|slot| format!("product-{slot:03}")),
    )
}

fn component_ids() -> BridgeSubscriptionReferenceWorkloadComponentIdSet {
    BridgeSubscriptionReferenceWorkloadComponentIdSet::from_declared_component_labels([
        "steel", "rubber", "copper", "glass", "labor",
    ])
}

fn lane_ids() -> BridgeSubscriptionReferenceWorkloadLaneIdSet {
    BridgeSubscriptionReferenceWorkloadLaneIdSet::from_declared_lane_labels([
        "authoritative-live",
        "historical-replay",
        "branch-local",
        "preview-discard",
    ])
}

fn sealed_certification_bundle(
    runtime: &RuntimeBridge,
    source_inputs: Vec<crate::facade::BridgeSubscriptionSourceArtifactInput>,
    rich_diagnostics_admitted: bool,
) -> crate::facade::BridgeSubscriptionCertificationBundleSealed {
    let manifest = runtime
        .declare_subscription_reference_workload_manifest(
            product_ids(),
            component_ids(),
            lane_ids(),
        )
        .expect("manifest should seal");
    let index = runtime.build_subscription_certification_source_index(source_inputs);
    let plan = runtime.plan_subscription_certification_bundle(&manifest, &index);
    let cost_profile = runtime
        .admit_subscription_certification_cost_profile(
            crate::facade::BridgeSubscriptionCertificationDensityPosture::SparseCertificationWindow,
            16,
            16,
            32,
            rich_diagnostics_admitted,
        )
        .expect("sparse certification profile should admit");
    let scratch = runtime.prepare_subscription_certification_scratch(&cost_profile);
    let draft = runtime
        .assemble_subscription_certification_bundle(plan, cost_profile, scratch)
        .expect("admitted certification bundle should assemble");
    runtime.seal_subscription_certification_bundle(draft)
}

fn active_source_inputs(
    declaration_role: SourceArtifactRole,
    strategy_role: SourceArtifactRole,
) -> Vec<crate::facade::BridgeSubscriptionSourceArtifactInput> {
    vec![
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::Declaration,
            declaration_role,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::AdmittedSubscription,
            SourceArtifactRole::Stable,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::ActiveDelivery,
            SourceArtifactRole::Stable,
        ),
        source_artifact(
            crate::facade::BridgeSubscriptionSourceArtifactKind::StrategyLowering,
            strategy_role,
        ),
    ]
}

fn source_artifact(
    artifact_kind: crate::facade::BridgeSubscriptionSourceArtifactKind,
    role: SourceArtifactRole,
) -> crate::facade::BridgeSubscriptionSourceArtifactInput {
    crate::facade::BridgeSubscriptionSourceArtifactInput::from_evidence(
        crate::facade::BridgeSubscriptionSourceArtifactEvidence::scenario(
            artifact_kind,
            crate::facade::BridgeSubscriptionSourceArtifactScenario::OfflineAudit,
            role,
        ),
    )
}

#[test]
fn offline_audit_diagnoses_from_canonicalized_sealed_bundles_and_reports() {
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
        active_source_inputs(SourceArtifactRole::Divergent, SourceArtifactRole::Stable),
        false,
    );
    let comparison_plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::ExpectedRejection,
            Some(crate::facade::BridgeSubscriptionCertificationFailureBoundary::RegistryDrift),
            None,
        )
        .expect("expected rejection comparison plan should admit");
    let comparison_report =
        runtime.compare_subscription_certification_bundles(comparison_plan, &left, &right);
    let bundle_index =
        runtime.build_subscription_offline_audit_bundle_index(vec![&right, &left, &left]);

    let audit_plan = runtime
        .plan_subscription_offline_audit(
            &bundle_index,
            vec![&comparison_report, &comparison_report],
            false,
            false,
        )
        .expect("offline audit plan should admit sealed bundle index and comparison reports");
    assert_eq!(
        audit_plan.comparison_report_count(),
        1,
        "audit planning must canonicalize duplicate comparison reports before summarizing outcomes"
    );
    assert_eq!(audit_plan.outcome_summary().expected_rejection_count(), 1);
    let audit_report = runtime.audit_subscription_certification_bundle_offline(audit_plan);
    let inspection = runtime.inspect_subscription_certification(&audit_report);

    assert_eq!(bundle_index.bundle_count(), 2);
    assert_eq!(
        audit_report.outcome(),
        crate::facade::BridgeSubscriptionOfflineAuditOutcome::DiagnosedOffline
    );
    assert_eq!(audit_report.comparison_report_count(), 1);
    assert_eq!(audit_report.outcome_summary().expected_rejection_count(), 1);
    assert_eq!(audit_report.outcome_summary().equivalent_count(), 0);
    assert_eq!(
        audit_report.counters().offline_audit_bundle_index_count(),
        1
    );
    assert_eq!(audit_report.counters().offline_audit_plan_count(), 1);
    assert_eq!(audit_report.counters().offline_audit_report_count(), 1);
    assert_eq!(audit_report.counters().offline_audit_bundle_count(), 2);
    assert_eq!(
        audit_report
            .counters()
            .offline_audit_comparison_report_count(),
        1
    );
    assert_eq!(inspection.audit_report_digest(), audit_report.digest());
    assert_eq!(inspection.outcome(), audit_report.outcome());
    assert_eq!(
        inspection.counter_digest(),
        audit_report.counters().digest().as_ref()
    );
    assert_eq!(
        inspection.outcome_summary_digest(),
        audit_report.outcome_summary().digest()
    );
    assert_eq!(inspection.host_log_dependency_count(), 0);
    assert_eq!(inspection.live_state_dependency_count(), 0);
}

#[test]
fn offline_audit_rejects_host_log_and_live_state_dependencies() {
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
    let comparison_plan = runtime
        .plan_subscription_certification_comparison(
            crate::facade::BridgeSubscriptionCertificationComparisonRelationship::SemanticEquivalence,
            None,
            None,
        )
        .expect("semantic equivalence comparison plan should admit");
    let comparison_report =
        runtime.compare_subscription_certification_bundles(comparison_plan, &left, &right);
    let bundle_index = runtime.build_subscription_offline_audit_bundle_index(vec![&left, &right]);

    let host_log_rejection = runtime
        .plan_subscription_offline_audit(&bundle_index, vec![&comparison_report], true, false)
        .expect_err("offline audit must reject host log dependencies");
    assert_eq!(
        host_log_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionOfflineAuditPlanRejectionKind::HostLogDependencyForbidden
    );
    assert_eq!(host_log_rejection.counters().host_log_dependency_count(), 1);

    let live_state_rejection = runtime
        .plan_subscription_offline_audit(&bundle_index, vec![&comparison_report], false, true)
        .expect_err("offline audit must reject live runtime dependencies");
    assert_eq!(
        live_state_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionOfflineAuditPlanRejectionKind::LiveStateDependencyForbidden
    );
    assert_eq!(
        live_state_rejection
            .counters()
            .live_state_dependency_count(),
        1
    );
}

#[test]
fn offline_audit_requires_bundles_and_comparison_reports() {
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
    let empty_index = runtime.build_subscription_offline_audit_bundle_index(Vec::new());
    let populated_index = runtime.build_subscription_offline_audit_bundle_index(vec![&left]);

    let empty_index_rejection = runtime
        .plan_subscription_offline_audit(&empty_index, Vec::new(), false, false)
        .expect_err("offline audit requires emitted sealed bundles");
    assert_eq!(
        empty_index_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionOfflineAuditPlanRejectionKind::EmptyBundleIndex
    );

    let missing_reports_rejection = runtime
        .plan_subscription_offline_audit(&populated_index, Vec::new(), false, false)
        .expect_err("offline audit requires comparison reports");
    assert_eq!(
        missing_reports_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionOfflineAuditPlanRejectionKind::MissingComparisonReports
    );
}
