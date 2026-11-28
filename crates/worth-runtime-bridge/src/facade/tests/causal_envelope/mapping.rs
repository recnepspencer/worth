use super::*;

#[test]
fn causal_envelope_denies_missing_preview_mapping_after_required_route_evidence() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-missing-preview"),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-preview",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-preview",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-preview",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeRoute,
                "missing-route-record",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing preview mapping should deny");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(denial.family(), BridgeCausalEvidenceFamily::BridgeRoute);
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_request_denies_duplicate_preview_references_before_mapping() {
    let duplicate_reference = bridge_reference(
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            BridgeCausalEvidenceFamily::BridgeRoute,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "preview-execution:duplicate-reference",
            ),
        )
        .expect("bridge reference identity should be valid"),
    );

    let denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:duplicate-preview",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:duplicate-preview",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:duplicate-preview",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            duplicate_reference.clone(),
            duplicate_reference,
        ],
    )
    .expect_err("duplicate preview references should deny before mapping");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::DuplicateEvidenceReference
    );
    assert_eq!(denial.family(), BridgeCausalEvidenceFamily::BridgeRoute);
    assert_eq!(
        denial.reference_identity_for_reporting(),
        "preview-execution:duplicate-reference"
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}
