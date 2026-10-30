use worth_foundational::{
    foundational_diagnostic_boundary_artifact_subject, BoundaryArtifactField, BoundaryArtifactId,
    FoundationalDiagnosticArtifactKind, FoundationalDiagnosticComparisonBundle,
};

fn main() {
    let subject = foundational_diagnostic_boundary_artifact_subject(
        BoundaryArtifactId::new(7),
        BoundaryArtifactField::Payload,
    );

    let _ = FoundationalDiagnosticComparisonBundle {
        left_artifact_kind: FoundationalDiagnosticArtifactKind::SupportReport,
        right_artifact_kind: FoundationalDiagnosticArtifactKind::ExplanationBundle,
        left_subject: subject.clone(),
        right_subject: subject,
        left_row_count: 1,
        right_row_count: 2,
        outcome: impossible_outcome(),
    };
}

fn impossible_outcome() -> ! {
    loop {
        std::hint::spin_loop();
    }
}
