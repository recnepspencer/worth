use super::*;

#[test]
fn common_changed_observation_plans_and_materializes_admitted() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-causal-dx-changed"),
            resource_request,
        )
        .unwrap();
    let plan = CausalInspection::for_test_observation(receipt_with_route(
        CausalObservationOutcome::Changed,
        routed.route_identity().bridge_admission_evidence(),
    ))
    .why_changed()
    .reference_only()
    .include_all_retained_evidence()
    .plan()
    .expect("changed observation should plan");

    assert_eq!(
        plan.support_posture(),
        CausalInspectionSupportPosture::Admitted
    );
    assert_eq!(plan.estimated_cost().bridge_envelope_assembly_count(), 1);

    let artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("admitted plan should materialize");

    assert!(artifact.is_admitted());
    assert_eq!(
        artifact.primary_result(),
        CausalInspectionArtifactKind::Admitted
    );
    assert_eq!(artifact.evidence_reference_count(), 2);
    assert_eq!(
        artifact.authority_bindings().len(),
        artifact.evidence().len()
    );
    assert!(artifact
        .authority_bindings()
        .iter()
        .any(|binding| binding.family() == "query_observation"));
    assert!(artifact
        .authority_bindings()
        .iter()
        .any(|binding| binding.family() == "bridge_route"));
}

#[test]
fn common_suppressed_observation_uses_reason_helper() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-causal-dx-suppressed"),
            resource_request,
        )
        .unwrap();
    let plan = CausalInspection::for_test_observation(receipt_with_route(
        CausalObservationOutcome::Suppressed,
        routed.route_identity().bridge_admission_evidence(),
    ))
    .why_suppressed()
    .reference_only()
    .plan()
    .expect("suppressed observation should plan through common path");

    let artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("suppressed plan should materialize");

    assert!(artifact.is_admitted());
    assert!(artifact.denial_reason().is_none());
}

#[test]
fn materialized_detail_common_path_is_advisory_before_bridge_materialization() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-causal-dx-advisory"),
            resource_request,
        )
        .unwrap();
    let plan = CausalInspection::for_test_observation(receipt_with_route(
        CausalObservationOutcome::Changed,
        routed.route_identity().bridge_admission_evidence(),
    ))
    .why_changed()
    .materialized_detail()
    .plan()
    .expect("materialized detail should plan as advisory");

    assert_eq!(
        plan.support_posture(),
        CausalInspectionSupportPosture::Advisory
    );
    assert_eq!(
        plan.explain().reason(),
        "materialized_detail_deferred_until_bridge_envelope"
    );

    let artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("advisory plan should materialize with advisory bridge summary");

    assert!(artifact.is_advisory());
    assert_eq!(
        artifact.advisory_reason(),
        Some("materialized_detail_deferred_until_bridge_envelope")
    );
    assert_eq!(artifact.warnings().len(), 1);
}

#[test]
fn common_path_preserves_core_digests_from_explicit_pipeline() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-causal-dx-parity"),
            resource_request,
        )
        .unwrap();
    let receipt = receipt_with_route(
        CausalObservationOutcome::Changed,
        routed.route_identity().bridge_admission_evidence(),
    );
    let plan = CausalInspection::for_test_observation(receipt.clone())
        .why_changed()
        .reference_only()
        .evidence_families([
            CausalEvidenceFamily::QueryInspection,
            CausalEvidenceFamily::BridgeRoute,
        ])
        .plan()
        .expect("common path should plan");

    let anchor = anchor_causal_observation(receipt, CausalInspectionReason::ChangedResult)
        .expect("explicit anchor should work");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(
            anchor,
            &[
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
            ],
        )
    else {
        panic!("explicit references should resolve");
    };
    let explicit_request = request_for_families(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
        &[
            CausalEvidenceFamily::QueryInspection,
            CausalEvidenceFamily::BridgeRoute,
        ],
    );
    let explicit_flow = admit_causal_inspection(explicit_request);
    let CausalInspectionProofFlow::Admitted(explicit_admitted) = explicit_flow else {
        panic!("explicit flow should admit");
    };

    assert_eq!(
        plan.anchor_for_reporting(),
        explicit_admitted.subject().anchor_for_reporting()
    );
    assert_eq!(
        plan.reference_set_digest(),
        explicit_admitted.subject().reference_set_digest()
    );
    assert_eq!(
        plan.request_for_reporting(),
        explicit_admitted.subject().request_digest()
    );
    assert_eq!(
        plan.admission_digest(),
        explicit_admitted.admitted_inspection_digest()
    );

    let summary =
        crate::runtime::tests::causal_inspection::bridge_admitted_summary(&explicit_admitted);
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    explicit_admitted
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
        .expect("explicit bridge envelope should assemble");
    let explicit_artifact = materialize_admitted_causal_inspection(
        &explicit_admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("explicit materialization should work");
    let common_artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("common materialization should work");

    assert_eq!(
        common_artifact.bridge_envelope_for_reporting(),
        explicit_artifact.bridge_envelope_for_reporting()
    );
    assert_eq!(
        common_artifact.artifact_for_reporting(),
        explicit_artifact.artifact_for_reporting()
    );
    assert_eq!(
        common_artifact.receipt().receipt_for_reporting(),
        explicit_artifact.receipt().receipt_for_reporting()
    );
}
