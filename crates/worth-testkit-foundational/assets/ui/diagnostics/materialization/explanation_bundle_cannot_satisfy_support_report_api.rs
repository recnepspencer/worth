use worth_foundational::{
    materialize_diagnostic_explanation_bundle, foundational_diagnostic_boundary_artifact_subject,
    foundational_diagnostic_code, foundational_diagnostic_locator_boundary_artifact,
    foundational_diagnostic_scope, AdmissionReadinessProfile, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, CertificationPostureProfile,
    CompatibilityPostureProfile, DiagnosticRichnessProfile, FoundationalDiagnosticCounterSnapshot,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticExplanationInput,
    FoundationalDiagnosticLocalityClaim, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticPartiality, FoundationalDiagnosticRow,
    FoundationalDiagnosticSemanticLabelSet, FoundationalDiagnosticSeverity,
    FoundationalDiagnosticSupportEvidencePosture, FoundationalDiagnosticSupportRow,
    FoundationalDiagnosticSurfaceAvailability, FoundationalDiagnosticWidenedFalloutPosture,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};

fn needs_support_report(_report: worth_foundational::FoundationalDiagnosticSupportReport) {}

fn main() {
    let report = materialize_diagnostic_explanation_bundle(
        FoundationalDiagnosticExplanationInput::new(
            foundational_diagnostic_boundary_artifact_subject(
                BoundaryArtifactId::new(1),
                BoundaryArtifactField::Payload,
            ),
            FoundationalDiagnosticOutcomeKind::Advisory,
            vec![FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
                foundational_diagnostic_code("support.row").unwrap(),
                foundational_diagnostic_scope("diagnostics.support").unwrap(),
                FoundationalDiagnosticSeverity::Advisory,
                foundational_diagnostic_boundary_artifact_subject(
                    BoundaryArtifactId::new(1),
                    BoundaryArtifactField::Payload,
                ),
                foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
                    BoundaryArtifactId::new(1),
                    BoundaryArtifactField::Payload,
                )),
                FoundationalDiagnosticOutcomeKind::Accepted,
                FoundationalDiagnosticSemanticLabelSet::new([]),
                FoundationalDiagnosticSupportEvidencePosture::Present(
                    worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
                ),
                FoundationalDiagnosticLocalityClaim::ExactSubject,
                FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
            ))],
            vec![],
            vec![],
            FoundationalDiagnosticSurfaceAvailability::retained_hot(),
            FoundationalDiagnosticPartiality::Complete,
            FoundationalDiagnosticCounterSnapshot::new(1, 0, 0, 0, 0, 0),
            vec![],
        ),
        FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: DiagnosticRichnessProfile::Standard,
            support_posture: SupportPostureProfile::SupportReady,
            compatibility_posture: CompatibilityPostureProfile::NativeOnly,
            admission_readiness: AdmissionReadinessProfile::Admitted,
            retention_delivery: RetentionDeliveryProfile::Retained,
            certification_posture: CertificationPostureProfile::Uncertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        })
        .unwrap(),
        FoundationalDiagnosticDeliveryClass::MustBeHot,
    )
    .unwrap();
    needs_support_report(report);
}
