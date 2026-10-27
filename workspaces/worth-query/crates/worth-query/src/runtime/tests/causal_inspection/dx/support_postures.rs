use super::*;

#[test]
fn support_discovery_names_supported_advisory_and_deferred_lanes() {
    let support = CausalInspection::support();
    let explanation = support.explain();

    assert_eq!(explanation.supported_row_count(), 1);
    assert_eq!(explanation.advisory_row_count(), 1);
    assert_eq!(explanation.deferred_row_count(), 2);
    assert!(support.rows().iter().any(|row| {
        row.explanation_family() == CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation
            && row.default_richness() == CausalInspectionRichness::ReferenceOnly
            && row.posture() == CausalInspectionSupportRowPosture::Supported
    }));
    assert!(support
        .rows()
        .iter()
        .any(|row| row.note().contains("later-milestone debt")));
}

#[test]
fn unsupported_durable_family_denies_without_bridge_assembly() {
    let runtime = bridge_runtime();
    let plan = crate::runtime::tests::causal_inspection::inspect_fixture(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:durable-denied",
                ),
            )],
        ),
    )
    .why_changed()
    .durable_archive()
    .plan()
    .expect("durable causal archive should reach Query denial");

    assert_eq!(
        plan.support_posture(),
        CausalInspectionSupportPosture::Denied
    );
    assert_eq!(plan.estimated_cost().bridge_envelope_assembly_count(), 0);
    assert_eq!(plan.estimated_cost().anchor_derivation_count(), 1);
    assert_eq!(
        plan.estimated_cost().evidence_reference_resolution_count(),
        1
    );
    assert_eq!(plan.estimated_cost().admission_count(), 1);
    assert_eq!(
        plan.redaction_policy(),
        CausalInspectionRedactionPolicy::PreserveDetail
    );
    assert_eq!(
        plan.materialization_policy(),
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact
    );
    assert_eq!(
        plan.explanation_family(),
        CausalInspectionExplanationFamily::DurableCausalArchive
    );
    assert_eq!(
        plan.requested_richness(),
        CausalInspectionRichness::ReferenceOnly
    );

    let artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("Query denial materializes without bridge assembly");

    assert!(artifact.is_denied());
    assert_eq!(
        artifact.denial_reason(),
        Some("unsupported_explanation_family")
    );
    assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 0);
    assert!(artifact.bridge_envelope_for_reporting().is_none());
}

#[test]
fn bridge_envelope_denial_materializes_denied_artifact_with_bridge_fields() {
    let runtime = bridge_runtime();
    let plan = crate::runtime::tests::causal_inspection::inspect_fixture(receipt_with_route(
        CausalObservationOutcome::Changed,
        crate::runtime::tests::causal_inspection::bridge_external_evidence(
            "route:causal-dx-missing",
        ),
    ))
    .why_changed()
    .reference_only()
    .plan()
    .expect("missing retained route still admits at Query boundary");

    let artifact = plan
        .materialize_with_bridge(&runtime)
        .expect("bridge denial should become a denied Query artifact");

    assert!(artifact.is_denied());
    assert_eq!(artifact.denial_reason(), Some("bridge_envelope_denial"));
    assert!(artifact
        .decision_trace()
        .bridge_denial_for_reporting()
        .is_some());
    assert_eq!(artifact.performance().bridge_envelope_assembly_count(), 1);
}
