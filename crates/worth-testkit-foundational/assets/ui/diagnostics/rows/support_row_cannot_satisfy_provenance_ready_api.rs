use worth_foundational::{
    foundational_diagnostic_boundary_artifact_subject, foundational_diagnostic_code,
    foundational_diagnostic_locator_boundary_artifact, foundational_diagnostic_scope,
    BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalDiagnosticLocalityClaim, FoundationalDiagnosticOutcomeKind,
    FoundationalDiagnosticProvenanceReadyRow, FoundationalDiagnosticSemanticLabelSet,
    FoundationalDiagnosticSeverity, FoundationalDiagnosticSupportEvidencePosture,
    FoundationalDiagnosticSupportRow, FoundationalDiagnosticWidenedFalloutPosture,
};

fn needs_provenance_ready(_row: FoundationalDiagnosticProvenanceReadyRow) {}

fn main() {
    let row = FoundationalDiagnosticSupportRow::new(
        foundational_diagnostic_code("support.present").unwrap(),
        foundational_diagnostic_scope("diagnostics.support").unwrap(),
        FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_boundary_artifact_subject(
            BoundaryArtifactId::new(9),
            BoundaryArtifactField::Payload,
        ),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(9),
            BoundaryArtifactField::Payload,
        )),
        FoundationalDiagnosticOutcomeKind::Partial,
        FoundationalDiagnosticSemanticLabelSet::new([]),
        FoundationalDiagnosticSupportEvidencePosture::Absent(
            worth_foundational::FoundationalDiagnosticAbsenceCause::Redacted,
        ),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    needs_provenance_ready(row);
}
