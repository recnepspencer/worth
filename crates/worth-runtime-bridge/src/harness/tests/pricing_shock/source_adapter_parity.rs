use super::support::*;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::source::SourceDeclarationIdentity;
use crate::source::SourceMaterializationRecord;
use worth_harness::facade::{ExecutionRequest, HarnessAdapter};

#[derive(Clone, Debug, PartialEq, Eq)]
struct PricingSourceProbeEvidence {
    materialized_record: SourceMaterializationRecord,
    retained_failure_count: usize,
}

impl PricingSourceProbeEvidence {
    fn assert_successful_materialization(&self) {
        assert_eq!(self.retained_failure_count, 0);
        assert_eq!(
            self.materialized_record
                .counters()
                .source_materialization_count(),
            1
        );
    }
}

fn execute_pricing_harness_source_probe(
    policy: BridgeRuntimePolicy,
    profile: ExecutionProfile,
    request_name: &str,
) -> PricingSourceProbeEvidence {
    let adapter = BridgeHarnessAdapter;
    let fixture = pricing_harness_fixture("bridge-pricing-harness-parity", policy);
    let mut runtime = adapter
        .create_runtime()
        .expect("pricing harness runtime should construct");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("pricing harness prepare should succeed");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("pricing harness fixture should load");
    adapter
        .execute(
            &mut runtime,
            &fixture,
            &ExecutionRequest::target(
                request_name,
                BridgeHarnessTargetId::source_materialize(
                    SourceDeclarationIdentity::admit_bridge_owned("source:pricing-main-history"),
                ),
            ),
            &profile,
        )
        .expect("pricing harness source probe should execute");
    let runtime_bridge = runtime
        .runtime
        .as_ref()
        .expect("pricing harness runtime should remain loaded");
    PricingSourceProbeEvidence {
        materialized_record: runtime_bridge
            .diagnostics()
            .last_source_materialization_record()
            .expect("pricing source probe should retain typed materialization record"),
        retained_failure_count: runtime_bridge.diagnostics().source_failure_records().len(),
    }
}

fn capture_pricing_bundle_with_harness_profile(
    policy: BridgeRuntimePolicy,
    preview_session_identity: BridgePreviewSessionIdentity,
    profile: ExecutionProfile,
    request_name: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    PricingWorkloadCertificationBundle,
    PricingSourceProbeEvidence,
) {
    (
        capture_pricing_workload_certification_bundle(
            policy,
            preview_session_identity,
            resource_request,
        ),
        execute_pricing_harness_source_probe(policy, profile, request_name),
    )
}

#[test]
fn pricing_shock_suite_25_through_27_parity_holds_across_direct_and_wrapped_source_adapter_shapes()
{
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (direct_bundle, direct_probe) = capture_pricing_bundle_with_harness_profile(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:adapter-direct"),
        ExecutionProfile::development("pricing-direct"),
        "pricing-source-direct",
        resource_request,
    );
    let (wrapped_bundle, wrapped_probe) = capture_pricing_bundle_with_harness_profile(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:adapter-direct"),
        ExecutionProfile::development("pricing-wrapped")
            .with_metadata("source_adapter_shape", "wrapped"),
        "pricing-source-wrapped",
        resource_request,
    );

    assert_eq!(
        direct_bundle.suite_25_artifact_json(),
        wrapped_bundle.suite_25_artifact_json()
    );
    assert_eq!(
        direct_bundle.suite_26_artifact_json(),
        wrapped_bundle.suite_26_artifact_json()
    );
    assert_eq!(
        direct_bundle.suite_27_artifact_json(),
        wrapped_bundle.suite_27_artifact_json()
    );
    assert_eq!(direct_bundle.digest(), wrapped_bundle.digest());

    let direct_export = direct_bundle.ml_pipeline_export_json();
    let wrapped_export = wrapped_bundle.ml_pipeline_export_json();
    assert_eq!(
        direct_export["lineage_provenance"]["causality"],
        wrapped_export["lineage_provenance"]["causality"]
    );
    assert_eq!(
        direct_export["lineage_provenance"]["historical_provenance"],
        wrapped_export["lineage_provenance"]["historical_provenance"]
    );
    direct_probe.assert_successful_materialization();
    wrapped_probe.assert_successful_materialization();
    assert_eq!(direct_probe, wrapped_probe);
    assert_eq!(
        direct_bundle.suite_25_digest_evidence().routing_digest,
        wrapped_bundle.suite_25_digest_evidence().routing_digest
    );
}

#[test]
fn pricing_shock_suite_25_through_27_parity_holds_across_source_and_policy_builder_load_orders() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let baseline_profile = ExecutionProfile::development("pricing-load-order-baseline");
    let (baseline_bundle, baseline_probe) = capture_pricing_bundle_with_harness_profile(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:load-order"),
        baseline_profile,
        "pricing-load-order-baseline",
        resource_request,
    );

    let variant_profiles = [
        ExecutionProfile::development("pricing-load-order-sources-first")
            .with_metadata("source_builder_load_order", "sources_first"),
        ExecutionProfile::development("pricing-load-order-sections-canonical")
            .with_metadata("policy_builder_load_order", "sections_canonical"),
        ExecutionProfile::development("pricing-load-order-sections-reverse")
            .with_metadata("policy_builder_load_order", "sections_reverse"),
        ExecutionProfile::development("pricing-load-order-combined")
            .with_metadata("source_builder_load_order", "sources_first")
            .with_metadata("policy_builder_load_order", "sections_reverse"),
    ];

    for (index, profile) in variant_profiles.into_iter().enumerate() {
        let (candidate_bundle, candidate_probe) = capture_pricing_bundle_with_harness_profile(
            BridgeRuntimePolicy::development(),
            BridgePreviewSessionIdentity::admit_bridge_owned("pricing:load-order"),
            profile,
            &format!("pricing-load-order-{index}"),
            resource_request,
        );

        assert_eq!(
            baseline_bundle.suite_25_artifact_json(),
            candidate_bundle.suite_25_artifact_json()
        );
        assert_eq!(
            baseline_bundle.suite_26_artifact_json(),
            candidate_bundle.suite_26_artifact_json()
        );
        assert_eq!(
            baseline_bundle.suite_27_artifact_json(),
            candidate_bundle.suite_27_artifact_json()
        );
        assert_eq!(baseline_bundle.digest(), candidate_bundle.digest());
        assert_eq!(
            baseline_bundle.trust_attack_matrix_json(),
            candidate_bundle.trust_attack_matrix_json()
        );
        assert_eq!(
            baseline_bundle.diagnostics_entrypoint_evidence(),
            candidate_bundle.diagnostics_entrypoint_evidence()
        );
        assert_eq!(
            baseline_bundle.certification_counter_evidence(),
            candidate_bundle.certification_counter_evidence()
        );
        baseline_probe.assert_successful_materialization();
        candidate_probe.assert_successful_materialization();
        assert_eq!(baseline_probe, candidate_probe);
    }
}
