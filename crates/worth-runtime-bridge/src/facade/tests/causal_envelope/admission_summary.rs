use super::{bridge_route_reference, query_observation_reference, runtime, BridgeRuntimePolicy};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceReferenceIdentity,
    BridgeCausalInspectionAdmissionSummary, BridgeCausalInspectionAdmissionSummaryKind,
};

#[test]
fn causal_envelope_request_carries_advisory_query_admission_summary() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-advisory-summary"),
            execution,
        )
        .expect("route should succeed");
    let admission_summary = BridgeCausalInspectionAdmissionSummary::advisory(
        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
            "query-admission:advisory",
        ),
        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
            "anchor:advisory",
        ),
    )
    .expect("advisory summary should be valid");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        admission_summary,
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:advisory",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
        ],
    )
    .expect("request should be valid");

    assert_eq!(
        request.admission_summary().kind(),
        BridgeCausalInspectionAdmissionSummaryKind::Advisory
    );
    assert_eq!(
        request.query_admission_for_reporting(),
        "query-admission:advisory"
    );
    assert_eq!(
        request.causal_observation_anchor_for_reporting(),
        "anchor:advisory"
    );
    assert!(request
        .admission_summary()
        .summary_for_reporting()
        .starts_with("worth.runtime.bridge.causal-envelope-identity.v1:"));
    let admission_summary_digest = request
        .admission_summary()
        .summary_for_reporting()
        .to_string();

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("advisory query admission summary should assemble");

    assert_eq!(
        envelope.admission_summary_kind(),
        BridgeCausalInspectionAdmissionSummaryKind::Advisory
    );
    assert_eq!(
        envelope.admission_summary_for_reporting(),
        admission_summary_digest
    );
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 1);
    assert_eq!(envelope.counters().external_authority_reference_count(), 1);
}
