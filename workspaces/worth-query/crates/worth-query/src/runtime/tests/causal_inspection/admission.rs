use super::super::super::*;

fn changed_reference_set() -> CausalEvidenceReferenceSet {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "admission-query-inspection-reference",
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "admission-bridge-route-reference",
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "admission-signal-invalidation-reference",
                    ),
                ),
            ],
        ),
        CausalInspectionReason::ChangedResult,
    )
    .unwrap();

    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(
            anchor,
            &[
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::SignalInvalidation,
            ],
        )
    else {
        panic!("expected changed fixture references to resolve");
    };
    reference_set
}

fn target_for(reference_set: &CausalEvidenceReferenceSet) -> CausalInspectionTarget {
    let receipt = reference_set.anchor().observation_receipt();
    causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .unwrap()
}

fn request(
    richness: CausalInspectionRichness,
    explanation_family: CausalInspectionExplanationFamily,
) -> CausalInspectionRequest {
    let reference_set = changed_reference_set();
    let target = target_for(&reference_set);
    request_causal_inspection(
        reference_set,
        target,
        explanation_family,
        richness,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .unwrap()
}

#[test]
fn causal_inspection_admission_success_carries_reference_permissions_and_trace() {
    let request = request(
        CausalInspectionRichness::ReferenceOnly,
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
    );
    let request_digest = request.request_for_reporting().to_string();
    let flow = admit_causal_inspection(request);

    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("expected reference-only cross-runtime inspection to admit");
    };

    assert_eq!(admitted.subject().request_digest(), request_digest);
    assert_eq!(
        admitted.decision().kind(),
        CausalInspectionAdmissionDecisionKind::Success
    );
    assert_eq!(
        admitted.decision().admitted_richness(),
        CausalInspectionRichness::ReferenceOnly
    );
    assert_eq!(
        admitted.decision().permitted_evidence_families(),
        &[CausalEvidenceFamily::BridgeRoute]
    );
    assert!(admitted
        .decision_trace()
        .row_for_key("explanation_family")
        .is_some());
    assert!(admitted
        .decision_trace()
        .row_for_key("evidence_family_scope")
        .is_some());
    assert!(!admitted.admitted_inspection_digest().is_empty());
    assert_eq!(
        admitted.receipt().decision_trace_index_for_reporting(),
        admitted.decision_trace().trace_for_reporting()
    );
    assert_eq!(admitted.counters().causal_inspection_request_count(), 1);
    assert_eq!(admitted.counters().causal_inspection_admission_count(), 1);
    assert_eq!(admitted.counters().causal_inspection_advisory_count(), 0);
    assert_eq!(admitted.counters().causal_inspection_denial_count(), 0);
    assert_eq!(
        admitted.counters().causal_decision_trace_lookup_count(),
        admitted.decision_trace().rows().len()
    );
    assert_eq!(
        admitted.counters().causal_decision_trace_index_hit_count(),
        admitted.decision_trace().rows().len()
    );
    assert_eq!(
        admitted.counters().bridge_causal_envelope_request_count(),
        0
    );
}

#[test]
fn causal_inspection_advisory_narrows_materialized_detail_before_bridge_envelope() {
    let flow = admit_causal_inspection(request(
        CausalInspectionRichness::MaterializedDetail,
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
    ));

    let CausalInspectionProofFlow::Advisory(advisory) = flow else {
        panic!("expected materialized detail to narrow into advisory");
    };

    assert_eq!(
        advisory.decision().kind(),
        CausalInspectionAdmissionDecisionKind::Advisory
    );
    assert_eq!(
        advisory.decision().advisory_kind(),
        Some(CausalInspectionAdvisoryKind::MaterializedDetailDeferredUntilBridgeEnvelope)
    );
    assert_eq!(
        advisory.decision().admitted_richness(),
        CausalInspectionRichness::ReferenceOnly
    );
    assert_eq!(
        advisory
            .decision_trace()
            .row_for_key("richness_policy")
            .unwrap()
            .decision(),
        CausalInspectionAdmissionDecisionKind::Advisory
    );
    assert!(!advisory.advisory_inspection_digest().is_empty());
    assert_eq!(advisory.counters().causal_inspection_advisory_count(), 1);
    assert_eq!(advisory.counters().causal_inspection_denial_count(), 0);
    assert_eq!(
        advisory.counters().bridge_causal_envelope_request_count(),
        0
    );
}

#[test]
fn causal_inspection_denies_unsupported_explanation_family_before_envelope_request() {
    let flow = admit_causal_inspection(request(
        CausalInspectionRichness::ReferenceOnly,
        CausalInspectionExplanationFamily::DurableCausalArchive,
    ));

    let CausalInspectionProofFlow::Denied(denied) = flow else {
        panic!("expected unsupported explanation family to deny");
    };

    assert_eq!(
        denied.decision().kind(),
        CausalInspectionAdmissionDecisionKind::Violation
    );
    assert_eq!(
        denied.decision().violation_kind(),
        Some(CausalInspectionViolationKind::UnsupportedExplanationFamily)
    );
    assert_eq!(
        denied
            .decision_trace()
            .row_for_key("explanation_family")
            .unwrap()
            .decision(),
        CausalInspectionAdmissionDecisionKind::Violation
    );
    assert!(!denied.denied_inspection_digest().is_empty());
    assert_eq!(denied.counters().causal_inspection_denial_count(), 1);
    assert_eq!(denied.counters().bridge_causal_envelope_request_count(), 0);
}

#[test]
fn causal_inspection_request_denies_target_and_unresolved_family_mismatches() {
    let reference_set = changed_reference_set();
    let unrelated =
        QueryObservationReceipt::fixture(CausalObservationOutcome::Suppressed, Vec::new());
    let bad_target = causal_inspection_target(
        unrelated.observation_target().clone(),
        reference_set
            .anchor()
            .observation_receipt()
            .result_shape_context()
            .clone(),
    )
    .unwrap();
    let target_mismatch = request_causal_inspection(
        reference_set.clone(),
        bad_target,
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
        CausalInspectionRichness::ReferenceOnly,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .unwrap_err();

    assert_eq!(
        target_mismatch.kind(),
        CausalInspectionRequestErrorKind::TargetObservationMismatch
    );
    assert!(!target_mismatch.failure_for_reporting().is_empty());

    let missing_family = request_causal_inspection(
        reference_set.clone(),
        target_for(&reference_set),
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
        CausalInspectionRichness::ReferenceOnly,
        &[CausalEvidenceFamily::SignalEvaluation],
    )
    .unwrap_err();

    assert_eq!(
        missing_family.kind(),
        CausalInspectionRequestErrorKind::RequestedEvidenceFamilyMissing
    );
}
