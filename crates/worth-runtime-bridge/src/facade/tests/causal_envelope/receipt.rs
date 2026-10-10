use super::{bridge_route_reference, external_reference, runtime, BridgeRuntimePolicy};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReferenceIdentity,
};

#[test]
fn causal_envelope_identity_and_receipt_bind_the_sealed_bridge_result() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-receipt"),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:receipt",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:receipt",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            external_reference(
                BridgeCausalEvidenceOwner::Query,
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:receipt",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            external_reference(
                BridgeCausalEvidenceOwner::Relational,
                BridgeCausalEvidenceReferenceIdentity::relational_authority(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "relational-authority:receipt",
                    ),
                )
                .expect("relational reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-invalidation:receipt",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("envelope should assemble");

    assert_eq!(
        envelope.identity().request_for_reporting(),
        envelope.request_for_reporting()
    );
    assert_eq!(
        envelope
            .identity()
            .causal_observation_anchor_for_reporting(),
        envelope.causal_observation_anchor_for_reporting()
    );
    assert_eq!(
        envelope.identity().counter_for_reporting(),
        envelope.counters().counter_for_reporting()
    );
    assert_eq!(
        envelope.receipt().envelope_identity_for_reporting(),
        envelope.identity().envelope_identity_for_reporting()
    );
    assert_eq!(
        envelope.receipt().envelope_for_reporting(),
        envelope.envelope_for_reporting()
    );
    assert_eq!(
        envelope.receipt().counter_for_reporting(),
        envelope.counters().counter_for_reporting()
    );
    assert!(!envelope
        .identity()
        .evidence_binding_digest_for_reporting()
        .is_empty());
    assert!(!envelope.receipt().receipt_for_reporting().is_empty());
    assert_eq!(envelope.bindings().len(), 4);
    assert_eq!(envelope.counters().lower_runtime_family_count(), 3);
    assert_eq!(envelope.counters().materialized_detail_count(), 4);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_identity_is_stable_across_unrelated_retained_routes() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut identities = Vec::new();
    let mut receipts = Vec::new();

    for unrelated_routes in [0, 5, 10] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        for index in 0..unrelated_routes {
            runtime
                .route(
                    crate::truth_identity_fixtures::truth_commit_fixture(format!(
                        "unrelated-causal-receipt-{index}"
                    )),
                    execution,
                )
                .expect("unrelated route should succeed");
        }
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(
                    "commit-causal-receipt-stable",
                ),
                execution,
            )
            .expect("target route should succeed");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:receipt-stable",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:receipt-stable",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                external_reference(
                    BridgeCausalEvidenceOwner::Query,
                    BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:receipt-stable",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target route should bind");

        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 1);
        assert_eq!(envelope.counters().lower_runtime_family_count(), 1);
        assert_eq!(envelope.counters().materialized_detail_count(), 2);
        identities.push(
            envelope
                .identity()
                .envelope_identity_for_reporting()
                .to_string(),
        );
        receipts.push(envelope.receipt().receipt_for_reporting().to_string());
    }

    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[1], identities[2]);
    assert_eq!(receipts[0], receipts[1]);
    assert_eq!(receipts[1], receipts[2]);
}
