use super::*;

#[test]
fn anchors_cover_changed_suppressed_denied_preview_and_replay_observations() {
    for (outcome, reason) in [
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
    ] {
        let receipt = QueryObservationReceipt::fixture(
            outcome,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        format!("query-inspection-{}", outcome.as_str()),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        format!("bridge-route-{}", outcome.as_str()),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        format!("signal-invalidation-{}", outcome.as_str()),
                    ),
                ),
            ],
        );

        let anchor = anchor_causal_observation(receipt, reason).unwrap();

        assert_eq!(anchor.observation_receipt().outcome(), outcome);
        assert_eq!(anchor.inspection_reason(), reason);
        assert_eq!(anchor.lower_runtime_evidence_family_count(), 3);
        assert_eq!(
            anchor.missing_reference_posture(),
            CausalObservationMissingReferencePosture::Complete
        );
        assert!(!anchor.anchor_digest().as_str().is_empty());
    }
}

#[test]
fn anchor_counters_are_exact_and_width_bound_to_carried_reference_families() {
    let receipt = QueryObservationReceipt::fixture(
        CausalObservationOutcome::Changed,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection-a",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection-b",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::RelationalAuthority,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "relational-authority-a",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-route-a",
                ),
            ),
        ],
    );

    let anchor = anchor_causal_observation(receipt, CausalInspectionReason::ChangedResult).unwrap();
    let counters = anchor.counters();

    assert_eq!(counters.source_receipt_family_count(), 1);
    assert_eq!(counters.reference_family_count(), 3);
    assert_eq!(
        counters.missing_reference_posture(),
        CausalObservationMissingReferencePosture::Complete
    );
    assert_eq!(
        counters.anchor_digest_width(),
        anchor.anchor_digest().as_str().len()
    );
    assert_eq!(counters.runtime_graph_scan_count(), 0);
    assert_eq!(counters.diagnostics_retention_scan_count(), 0);
    assert!(!counters.counter_snapshot().is_empty());
}

#[test]
fn anchor_derivation_denies_receipts_without_carried_evidence_identities() {
    let receipt = QueryObservationReceipt::fixture(CausalObservationOutcome::Changed, Vec::new());

    let error =
        anchor_causal_observation(receipt, CausalInspectionReason::ChangedResult).unwrap_err();

    assert_eq!(
        error.kind(),
        CausalObservationAnchorErrorKind::MissingRequiredEvidenceReference
    );
    assert!(error.message().contains("evidence identity"));
    assert!(!error.failure_digest().is_empty());
}

#[test]
fn anchor_derivation_seals_empty_fixture_labels_into_typed_reference_digests() {
    let receipt = QueryObservationReceipt::fixture(
        CausalObservationOutcome::Changed,
        vec![CausalObservationEvidenceIdentity::new(
            CausalEvidenceFamily::BridgeRoute,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(""),
        )],
    );

    let anchor = anchor_causal_observation(receipt, CausalInspectionReason::ChangedResult)
        .expect("typed fixture references should seal empty source labels");

    let carried_reference =
        anchor.observation_receipt().evidence_identities()[0].reference_digest();
    assert!(!carried_reference.as_str().is_empty());
}

#[test]
fn anchor_derivation_denies_inspection_reason_outcome_mismatch() {
    let receipt = QueryObservationReceipt::fixture(
        CausalObservationOutcome::Denied,
        vec![CausalObservationEvidenceIdentity::new(
            CausalEvidenceFamily::QueryInspection,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "denied-inspection",
            ),
        )],
    );

    let error =
        anchor_causal_observation(receipt, CausalInspectionReason::ChangedResult).unwrap_err();

    assert_eq!(
        error.kind(),
        CausalObservationAnchorErrorKind::InspectionReasonOutcomeMismatch
    );
    assert!(error.message().contains("reason must match"));
    assert!(!error.failure_digest().is_empty());
}
