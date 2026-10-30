use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceLineageSubject, FoundationalDiagnosticLocator,
    FoundationalTransitionLocator,
};

fn accepts_object_level_attachment(
    _: &worth_foundational::FoundationalBoundaryEvidenceObjectContinuityAttachment,
) {
}

fn main() {
    let bundle = boundary_evidence()
        .attachment()
        .for_transition(FoundationalTransitionLocator::CommitParentage(
            worth_foundational::FoundationalCommitParentageLocator::new(
                worth_foundational::FoundationalCommitId::new(BoundaryHandle::new(7)),
                worth_foundational::FoundationalCommitParentBasis::new(
                    worth_foundational::EquivalenceBasisId::new(8),
                ),
            ),
        ))
        .with_locator_continuity(
            FoundationalBoundaryEvidenceLineageSubject::new(BoundaryHandle::new(7)),
            FoundationalDiagnosticLocator::BoundaryArtifact(BoundaryArtifactLocator::new(
                BoundaryArtifactId::new(70),
                BoundaryArtifactField::Basis,
            )),
            FoundationalDiagnosticLocator::BoundaryArtifact(BoundaryArtifactLocator::new(
                BoundaryArtifactId::new(71),
                BoundaryArtifactField::Basis,
            )),
        );

    accepts_object_level_attachment(bundle.locator_continuity().unwrap());
}
