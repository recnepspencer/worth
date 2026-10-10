use super::super::super::*;
use super::materialization::support::*;

#[test]
fn cross_basis_pairing_denies_before_anchor_or_envelope_assembly() {
    let receipt = receipt_with_evidence(
        CausalObservationOutcome::Changed,
        &[(CausalEvidenceFamily::QueryInspection, "cross-basis")],
    );
    let foreign_basis = crate::basis_lifecycle::basis_lifecycle()
        .branch_head("foreign-branch", true)
        .inspect()
        .unwrap();

    let error = CausalInspection::for_observation(receipt, foreign_basis)
        .why_changed()
        .plan()
        .unwrap_err();

    assert_eq!(error.kind(), CausalInspectionPlanErrorKind::BasisMismatch);
    assert!(matches!(error, CausalInspectionPlanError::BasisMismatch(_)));
}

#[test]
fn reason_outcome_matrix_denies_every_mismatched_pair() {
    let rows = [
        (
            CausalObservationOutcome::Changed,
            CausalInspectionReason::ChangedResult,
        ),
        (
            CausalObservationOutcome::Suppressed,
            CausalInspectionReason::SuppressedResult,
        ),
        (
            CausalObservationOutcome::Denied,
            CausalInspectionReason::DeniedResult,
        ),
        (
            CausalObservationOutcome::BranchPreview,
            CausalInspectionReason::BranchPreviewResult,
        ),
        (
            CausalObservationOutcome::Replayed,
            CausalInspectionReason::HistoricalReplayResult,
        ),
    ];

    for (outcome, expected_reason) in rows {
        for (_, candidate_reason) in rows {
            let result = anchor_causal_observation(
                receipt_with_evidence(
                    outcome,
                    &[(CausalEvidenceFamily::QueryInspection, "query-inspection")],
                ),
                candidate_reason,
            );

            if candidate_reason == expected_reason {
                assert!(
                    result.is_ok(),
                    "expected {:?} to admit {:?}",
                    outcome,
                    candidate_reason
                );
            } else {
                let error = result.expect_err("mismatched reason/outcome should deny");
                assert_eq!(
                    error.kind(),
                    CausalObservationAnchorErrorKind::InspectionReasonOutcomeMismatch
                );
                assert!(!error.failure_digest().is_empty());
            }
        }
    }
}

#[test]
fn common_path_missing_evidence_matrix_fails_before_admission() {
    for missing_family in [
        CausalEvidenceFamily::BridgeRoute,
        CausalEvidenceFamily::RelationalAuthority,
        CausalEvidenceFamily::SignalInvalidation,
        CausalEvidenceFamily::SignalEvaluation,
        CausalEvidenceFamily::SignalReplayCursor,
    ] {
        let error = CausalInspection::for_test_observation(receipt_with_evidence(
            CausalObservationOutcome::Changed,
            &[(CausalEvidenceFamily::QueryInspection, "query-inspection")],
        ))
        .why_changed()
        .reference_only()
        .evidence_families([CausalEvidenceFamily::QueryInspection, missing_family])
        .plan()
        .expect_err("missing requested evidence should fail before admission");

        assert_eq!(error.kind(), CausalInspectionPlanErrorKind::MissingEvidence);
        assert!(!error.failure_digest().is_empty());
    }
}

#[test]
fn future_explanation_families_deny_without_bridge_assembly() {
    let runtime = bridge_runtime();

    for family_builder in [
        CausalInspection::durable_archive,
        CausalInspection::store_backed_replay,
    ] {
        let plan = family_builder(
            CausalInspection::for_test_observation(receipt_with_evidence(
                CausalObservationOutcome::Changed,
                &[(CausalEvidenceFamily::QueryInspection, "query-inspection")],
            ))
            .why_changed()
            .reference_only(),
        )
        .plan()
        .expect("future explanation family should produce a denied plan");

        assert_eq!(
            plan.support_posture(),
            CausalInspectionSupportPosture::Denied
        );
        assert_eq!(plan.estimated_cost().anchor_derivation_count(), 1);
        assert_eq!(
            plan.estimated_cost().evidence_reference_resolution_count(),
            1
        );
        assert_eq!(plan.estimated_cost().admission_count(), 1);
        assert_eq!(plan.estimated_cost().bridge_envelope_assembly_count(), 0);

        let artifact = plan
            .materialize_with_bridge(&runtime)
            .expect("denied future family should materialize without bridge assembly");
        assert!(artifact.is_denied());
        assert!(artifact.bridge_envelope_for_reporting().is_none());
        assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 0);
        assert_eq!(
            artifact.denial_reason(),
            Some("unsupported_explanation_family")
        );
    }
}

#[test]
fn redaction_and_materialization_policy_matrix_preserves_causal_identity() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::causal_truth_commit_identity("commit-causal-adversarial-policy"),
            resource_request,
        )
        .unwrap();
    let mut causal_identity_digest = None;
    let mut policy_digests = Vec::new();
    let mut artifact_digests = Vec::new();

    let route_identity = routed.route_identity().bridge_admission_evidence();
    for redaction_policy in [
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionRedactionPolicy::DigestOnly,
    ] {
        for materialization_policy in [
            CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            CausalInspectionMaterializationPolicy::DigestReferenceOnly,
        ] {
            let receipt = QueryObservationReceipt::fixture(
                CausalObservationOutcome::Changed,
                vec![
                    CausalObservationEvidenceIdentity::new(
                        CausalEvidenceFamily::QueryInspection,
                        crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                            "query-inspection",
                        ),
                    ),
                    CausalObservationEvidenceIdentity::new(
                        CausalEvidenceFamily::BridgeRoute,
                        route_identity.clone(),
                    ),
                ],
            );
            let plan = CausalInspection::for_test_observation(receipt)
                .why_changed()
                .reference_only()
                .redaction(redaction_policy)
                .materialization(materialization_policy)
                .plan()
                .expect("policy matrix row should plan");
            let artifact = plan
                .materialize_with_bridge(&runtime)
                .expect("policy matrix row should materialize");

            assert!(artifact.is_admitted());
            assert_eq!(artifact.denial_reason(), None);
            assert!(!artifact.receipt().policy_for_reporting().is_empty());
            assert_eq!(
                artifact
                    .evidence()
                    .iter()
                    .all(|evidence| evidence.detail_redacted()),
                redaction_policy == CausalInspectionRedactionPolicy::DigestOnly
            );

            match causal_identity_digest {
                Some(ref digest) => assert_eq!(artifact.causal_identity_for_reporting(), digest),
                None => {
                    causal_identity_digest =
                        Some(artifact.causal_identity_for_reporting().to_string())
                }
            }
            policy_digests.push(artifact.receipt().policy_for_reporting().to_string());
            artifact_digests.push(artifact.artifact_for_reporting().to_string());
        }
    }

    policy_digests.sort();
    policy_digests.dedup();
    artifact_digests.sort();
    artifact_digests.dedup();
    assert_eq!(policy_digests.len(), 4);
    assert_eq!(artifact_digests.len(), 4);
}

fn receipt_with_evidence(
    outcome: CausalObservationOutcome,
    evidence: &[(CausalEvidenceFamily, &str)],
) -> QueryObservationReceipt {
    QueryObservationReceipt::fixture(
        outcome,
        evidence
            .iter()
            .map(|(family, digest)| {
                CausalObservationEvidenceIdentity::new(
                    *family,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(digest),
                )
            })
            .collect(),
    )
}
