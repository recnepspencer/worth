use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceSourceBasis,
};

fn main() {
    let reconstructed = boundary_evidence()
        .lineage()
        .reconstructed_equivalence(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(2),
        ))
        .with_provenance(
            boundary_evidence()
                .provenance()
                .replay_derived(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                    BoundaryArtifactLocator::new(BoundaryArtifactId::new(2), BoundaryArtifactField::Basis),
                ))
                .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
                .unwrap_success(),
        )
        .unwrap_success();

    let _ = reconstructed.restoration_receipt();
}

trait UnwrapSuccess<T> {
    fn unwrap_success(self) -> T;
}

impl<T, E> UnwrapSuccess<T> for worth_proof::TransitionOutcome<T, E> {
    fn unwrap_success(self) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected success"),
        }
    }
}
