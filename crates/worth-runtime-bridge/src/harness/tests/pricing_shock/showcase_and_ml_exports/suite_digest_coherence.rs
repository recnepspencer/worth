use super::super::support::*;

#[test]
fn pricing_shock_suite_artifacts_and_showcase_digests_are_semantically_coherent() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-suite-coherence"),
        resource_request,
    );
    let artifact = bundle.showcase_artifact_json();
    let export = bundle.ml_pipeline_export_json();
    let suite_25 = bundle.suite_25_digest_evidence();
    let suite_26 = bundle.suite_26_digest_evidence();
    let suite_27 = bundle.suite_27_digest_evidence();

    assert_eq!(
        artifact["demo_artifact_family"]["control_digest"]
            .as_str()
            .expect("control digest should export as a string"),
        suite_25.reference_workload_bundle_digest
    );
    assert_eq!(
        artifact["demo_artifact_family"]["hostile_digest"]
            .as_str()
            .expect("hostile digest should export as a string"),
        suite_26.reference_workload_failure_bundle_digest
    );
    assert_eq!(
        artifact["demo_artifact_family"]["certification_digest"]
            .as_str()
            .expect("certification digest should export as a string"),
        suite_27.certification_bundle_digest
    );
    assert_eq!(
        artifact["demo_artifact_family"]["showcase_digest"]
            .as_str()
            .expect("showcase digest should export as a string"),
        export["bundle_digest"]
            .as_str()
            .expect("export digest should be a string")
    );
    assert_eq!(
        export["lineage_provenance"]["causality"]["suite_25_causality_digest"]
            .as_str()
            .expect("causality digest should export as a string"),
        suite_25.causality_digest
    );
    assert_eq!(
        export["lineage_provenance"]["causality"]["suite_25_routing_digest"]
            .as_str()
            .expect("routing digest should export as a string"),
        suite_25.routing_digest
    );
    assert_eq!(
        export["lineage_provenance"]["causality"]["suite_25_replay_digest"]
            .as_str()
            .expect("replay digest should export as a string"),
        suite_25.replay_digest
    );
    assert_eq!(
        export["suite_27"]["reference_workload_bundle_digest"]
            .as_str()
            .expect("suite 27 reference digest should export as a string"),
        suite_25.reference_workload_bundle_digest
    );
    assert_ne!(
        suite_25.reference_workload_bundle_digest,
        suite_26.reference_workload_failure_bundle_digest
    );
    assert_ne!(
        suite_25.reference_workload_bundle_digest,
        suite_27.certification_bundle_digest
    );
    assert_ne!(
        suite_26.reference_workload_failure_bundle_digest,
        suite_27.certification_bundle_digest
    );
}
