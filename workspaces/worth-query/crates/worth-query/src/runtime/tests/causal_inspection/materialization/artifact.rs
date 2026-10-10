use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceReferenceIdentity, BridgeCausalInspectionAdmissionSummary,
    BridgeIdentityEvidence,
};

use super::super::super::super::*;
use super::support::*;

#[test]
fn admitted_query_causal_artifact_materializes_sealed_bridge_envelope() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-materialization"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("reference-only cross-runtime inspection should admit");
    };
    let summary = BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("query admission summary should be valid");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");

    let artifact = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("admitted materialization should consume bridge envelope");
    let QueryCausalInspectionArtifact::Admitted(artifact) = artifact else {
        panic!("expected admitted query causal artifact");
    };

    assert_eq!(
        artifact.query_observation_for_reporting(),
        admitted.subject().query_observation_for_reporting()
    );
    assert_eq!(
        artifact.result_shape_context_for_reporting(),
        admitted.subject().result_shape_context_for_reporting()
    );
    assert_eq!(
        artifact.bridge_envelope_for_reporting(),
        crate::runtime::tests::causal_test_compose_bridge_causal_explanation_envelope_identity_for_reporting(
            &envelope
        )
    );
    assert_eq!(
        artifact.readmission_proof().query_admission_for_reporting(),
        admitted.admitted_inspection_digest()
    );
    assert_eq!(
        artifact.readmission_proof().anchor_for_reporting(),
        admitted.subject().anchor_for_reporting()
    );
    assert_eq!(
        artifact.readmission_proof().bridge_envelope_for_reporting(),
        crate::runtime::tests::causal_test_compose_bridge_causal_envelope_identity_for_reporting(
            &envelope
        )
    );
    assert_eq!(
        artifact.bridge_readmission_proof_for_reporting(),
        artifact
            .readmission_proof()
            .readmission_proof_for_reporting()
    );
    let expected_bridge_receipt =
        crate::runtime::tests::causal_test_compose_bridge_causal_envelope_receipt_identity_for_reporting(
            envelope.receipt(),
        );
    assert_eq!(
        artifact.receipt().bridge_receipt_for_reporting(),
        Some(expected_bridge_receipt.as_str())
    );
    assert_eq!(artifact.evidence_references().len(), 2);
    assert!(artifact
        .evidence_references()
        .iter()
        .any(|reference| reference.retained_record_for_reporting().is_some()));
    assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 1);
    assert_eq!(artifact.performance().artifact_serialization_count(), 1);
    assert_eq!(artifact.performance().bridge_unindexed_scan_count(), 0);
    assert_eq!(
        artifact.boundary_categories().len(),
        6,
        "artifact should carry all boundary envelope categories"
    );
}

#[test]
fn advisory_query_causal_artifact_redacts_detail_without_changing_bridge_identity() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-advisory"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::MaterializedDetail,
    ));
    let CausalInspectionProofFlow::Advisory(advisory) = flow else {
        panic!("materialized detail should narrow to advisory");
    };
    let summary = BridgeCausalInspectionAdmissionSummary::advisory(
        bridge_query_evidence(
            "causal-inspection-outcome",
            advisory.advisory_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            advisory.subject().anchor_for_reporting(),
        ),
    )
    .expect("query advisory summary should be valid");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    advisory
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");

    let full_artifact = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("full advisory materialization should consume bridge envelope");
    let redacted_artifact = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::DigestOnly,
        CausalInspectionMaterializationPolicy::DigestReferenceOnly,
    )
    .expect("redacted advisory materialization should consume bridge envelope");

    assert_eq!(
        full_artifact.causal_identity_for_reporting(),
        redacted_artifact.causal_identity_for_reporting(),
        "redaction must not change query causal identity"
    );
    assert_ne!(
        full_artifact.artifact_for_reporting(),
        redacted_artifact.artifact_for_reporting(),
        "redaction should change materialized artifact detail"
    );

    let QueryCausalInspectionArtifact::Advisory(artifact) = redacted_artifact else {
        panic!("expected advisory query causal artifact");
    };

    assert_eq!(
        artifact.query_observation_for_reporting(),
        advisory.subject().query_observation_for_reporting()
    );
    assert_eq!(
        artifact.result_shape_context_for_reporting(),
        advisory.subject().result_shape_context_for_reporting()
    );
    assert_eq!(
        artifact.bridge_envelope_for_reporting(),
        crate::runtime::tests::causal_test_compose_bridge_causal_explanation_envelope_identity_for_reporting(
            &envelope
        )
    );
    assert_eq!(
        artifact.readmission_proof().query_admission_for_reporting(),
        advisory.advisory_inspection_digest()
    );
    assert_eq!(
        artifact.readmission_proof().anchor_for_reporting(),
        advisory.subject().anchor_for_reporting()
    );
    assert_eq!(
        artifact.bridge_readmission_proof_for_reporting(),
        artifact
            .readmission_proof()
            .readmission_proof_for_reporting()
    );
    assert!(artifact
        .evidence_references()
        .iter()
        .all(|reference| reference.detail_redacted()));
    assert!(artifact
        .evidence_references()
        .iter()
        .all(|reference| reference.retained_record_for_reporting().is_none()));
    assert_eq!(
        artifact.performance().redaction_count(),
        artifact.evidence_references().len()
    );
}

#[test]
fn admitted_materialization_rejects_wrong_bridge_summary_kind() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-summary-mismatch"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("reference-only cross-runtime inspection should admit");
    };
    let wrong_summary = BridgeCausalInspectionAdmissionSummary::advisory(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("mismatched bridge summary should still be structurally valid");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        wrong_summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");

    let error = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionMaterializationErrorKind::AdmissionSummaryKindMismatch
    );
}

fn bridge_query_evidence(scope: &str, token: &str) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_query_evidence(scope, token)
}
