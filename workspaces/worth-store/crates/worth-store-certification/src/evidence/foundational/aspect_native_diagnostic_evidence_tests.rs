use worth_foundational::{
    materialize_diagnostic_explanation_bundle, materialize_diagnostic_support_report,
    AdmissionReadinessProfile, BoundaryArtifactField, BoundaryArtifactId,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    ExecutionObjectiveProfile, FoundationalDiagnosticArtifactKind,
    FoundationalDiagnosticCounterSnapshot, FoundationalDiagnosticDeliveryClass,
    FoundationalDiagnosticExplanationInput, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticPartiality, FoundationalDiagnosticSubject,
    FoundationalDiagnosticSupportClaimStrength, FoundationalDiagnosticSupportInput,
    FoundationalDiagnosticSurfaceAvailability, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_store_aspect_native::{
    StoreDiagnosticExplanationBundleEvidence, StoreDiagnosticSupportReportEvidence,
    StorePhysicalBoundaryWitness,
};
use worth_store_contracts::{StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};

#[test]
fn store_diagnostic_evidence_requires_foundational_diagnostic_artifacts() {
    let support_report = foundational_diagnostic_support_report();
    let support_evidence =
        StoreDiagnosticSupportReportEvidence::new(support_report, physical_witness());
    let explanation_bundle = foundational_diagnostic_explanation_bundle();
    let explanation_evidence =
        StoreDiagnosticExplanationBundleEvidence::new(explanation_bundle, physical_witness());

    assert_eq!(
        support_evidence.diagnostic().artifact_kind(),
        FoundationalDiagnosticArtifactKind::SupportReport
    );
    assert_eq!(
        explanation_evidence.diagnostic().artifact_kind(),
        FoundationalDiagnosticArtifactKind::ExplanationBundle
    );
}

fn foundational_diagnostic_support_report(
) -> worth_foundational::FoundationalDiagnosticSupportReport {
    materialize_diagnostic_support_report(
        FoundationalDiagnosticSupportInput::new(
            diagnostic_subject(),
            FoundationalDiagnosticOutcomeKind::Accepted,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            FoundationalDiagnosticSurfaceAvailability::retained_hot(),
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            FoundationalDiagnosticPartiality::Complete,
            FoundationalDiagnosticCounterSnapshot::new(1, 0, 0, 0, 0, 0),
            Vec::new(),
        ),
        diagnostic_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .unwrap()
}

fn foundational_diagnostic_explanation_bundle(
) -> worth_foundational::FoundationalDiagnosticExplanationBundle {
    materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            diagnostic_subject(),
            FoundationalDiagnosticOutcomeKind::Accepted,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            FoundationalDiagnosticSurfaceAvailability::retained_hot(),
            FoundationalDiagnosticPartiality::Complete,
            FoundationalDiagnosticCounterSnapshot::new(1, 0, 0, 0, 0, 0),
            Vec::new(),
        ),
        diagnostic_profile(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .unwrap()
}

fn diagnostic_subject() -> FoundationalDiagnosticSubject {
    FoundationalDiagnosticSubject::BoundaryArtifact {
        artifact_locator: worth_foundational::BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(42),
            BoundaryArtifactField::Payload,
        ),
    }
}

fn diagnostic_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
    })
    .unwrap()
}

fn physical_witness() -> StorePhysicalBoundaryWitness {
    StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap()
}
