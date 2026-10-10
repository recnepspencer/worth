use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner, BridgeCausalEvidenceReferenceIdentity,
    BridgeCausalInspectionAdmissionSummary, BridgeIdentityEvidence,
};

use super::super::super::super::*;
use super::support::*;

#[test]
fn denied_query_causal_artifact_carries_boundary_context_without_bridge_envelope() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-denied"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("target should match receipt");
    let request = request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::DurableCausalArchive,
        CausalInspectionRichness::ReferenceOnly,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .expect("causal inspection request should reach admission boundary");
    let flow = admit_causal_inspection(request);
    let CausalInspectionProofFlow::Denied(denied) = flow else {
        panic!("durable causal archive should deny for phase-5 materialization");
    };

    let artifact = materialize_denied_causal_inspection(
        &denied,
        None,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    );
    let QueryCausalInspectionArtifact::Denied(artifact) = artifact else {
        panic!("expected denied query causal artifact");
    };

    assert_eq!(
        artifact.query_observation_for_reporting(),
        denied.subject().query_observation_for_reporting()
    );
    assert_eq!(
        artifact.result_shape_context_for_reporting(),
        denied.subject().result_shape_context_for_reporting()
    );
    assert_eq!(
        artifact.boundary_categories().len(),
        6,
        "denied artifact should remain a complete boundary envelope"
    );
    assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 0);
    assert_eq!(artifact.performance().materialization_count(), 1);
    assert_eq!(artifact.performance().artifact_serialization_count(), 1);
    assert!(!artifact.causal_identity_for_reporting().is_empty());
}

#[test]
fn denied_query_causal_artifact_carries_bridge_denial_posture_and_counters() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-bridge-denied"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("target should match receipt");
    let request = request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::DurableCausalArchive,
        CausalInspectionRichness::ReferenceOnly,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .expect("causal inspection request should reach admission boundary");
    let CausalInspectionProofFlow::Denied(denied) = admit_causal_inspection(request) else {
        panic!("durable causal archive should deny before materialization");
    };
    let summary = BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            denied.denied_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            denied.subject().anchor_for_reporting(),
        ),
    )
    .expect("query denial summary should be syntactically valid for bridge denial fixture");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    denied
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    bridge_evidence("signal-invalidation:missing-route-fixture"),
                )
                .expect("signal invalidation reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid enough to reach assembly denial");
    let bridge_denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect_err("bridge envelope must require route evidence");

    assert_eq!(
        bridge_denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRequiredBridgeRouteEvidence
    );

    let artifact = materialize_denied_causal_inspection(
        &denied,
        Some(&bridge_denial),
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    );
    let QueryCausalInspectionArtifact::Denied(artifact) = artifact else {
        panic!("expected denied query causal artifact");
    };

    let expected_bridge_denial =
        crate::runtime::tests::causal_test_compose_bridge_causal_denial_for_reporting(
            &bridge_denial,
        );
    assert_eq!(
        artifact.bridge_denial_for_reporting(),
        Some(expected_bridge_denial.as_str())
    );
    assert_eq!(
        artifact.bridge_denial_kind(),
        Some("missing_required_bridge_route_evidence")
    );
    assert_eq!(artifact.bridge_denial_family(), Some("bridge_route"));
    assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 1);
    assert_eq!(artifact.performance().bridge_binding_count(), 2);
    assert_eq!(artifact.performance().bridge_lookup_count(), 0);
    assert_eq!(artifact.performance().materialized_detail_count(), 2);
}

fn bridge_evidence(value: impl AsRef<str>) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_external_evidence(value)
}

fn bridge_query_evidence(scope: &str, token: &str) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_query_evidence(scope, token)
}
