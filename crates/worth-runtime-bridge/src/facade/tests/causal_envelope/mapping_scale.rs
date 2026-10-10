use super::mapping_support::{
    bridge_preview_execution_reference, bridge_route_reference, preview_declaration,
    query_observation_reference,
};
use super::{runtime, BridgeRuntimePolicy};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceReferenceIdentity,
    BridgePreviewSessionDeclarationIdentity, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBindingIdentity,
};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn causal_envelope_preview_mapping_cost_ignores_unrelated_preview_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_previews in [0, 3, 9] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        for index in 0..unrelated_previews {
            let admitted = runtime
                .admit_preview_session(
                    BridgePreviewSessionIdentity::admit_bridge_owned(format!(
                        "preview-session:noise-{index}"
                    )),
                    preview_declaration(
                        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(format!(
                            "preview:noise-{index}"
                        )),
                        BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(format!(
                            "binding:noise-{index}"
                        )),
                        crate::truth_identity_fixtures::truth_branch_fixture(format!(
                            "truth:noise-{index}"
                        )),
                        BridgeSignalBranchIdentity::admit_bridge_owned(format!(
                            "signal:noise-{index}"
                        )),
                        crate::truth_identity_fixtures::truth_snapshot_fixture(format!(
                            "snapshot:noise-{index}"
                        )),
                    ),
                )
                .expect("unrelated preview should admit");
            runtime.activate_preview_session(admitted, 1, 0, 0);
        }
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-preview-scale"),
                execution,
            )
            .expect("route should succeed");
        let admitted = runtime
            .admit_preview_session(
                BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:causal-scale"),
                preview_declaration(
                    BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                        "preview:causal-scale",
                    ),
                    BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                        "binding:causal-scale",
                    ),
                    crate::truth_identity_fixtures::truth_branch_fixture("truth:causal-scale"),
                    BridgeSignalBranchIdentity::admit_bridge_owned("signal:causal-scale"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:causal-scale"),
                ),
            )
            .expect("target preview should admit");
        let (_, execution_record) = runtime.activate_preview_session(admitted, 2, 1, 1);
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:preview-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:preview-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:preview-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_preview_execution_reference(&execution_record),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target preview should bind");

        assert_eq!(
            runtime.diagnostics().preview_execution_records().len(),
            unrelated_previews + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
        assert_eq!(envelope.counters().external_authority_reference_count(), 1);
        assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
        envelope_identities.push(
            envelope
                .identity()
                .envelope_identity_for_reporting()
                .to_string(),
        );
    }

    assert_eq!(envelope_identities[0], envelope_identities[1]);
    assert_eq!(envelope_identities[1], envelope_identities[2]);
}
