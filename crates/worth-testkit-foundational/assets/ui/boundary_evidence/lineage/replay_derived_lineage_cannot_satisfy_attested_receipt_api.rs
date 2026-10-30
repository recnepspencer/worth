use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceSourceBasis,
};

fn main() {
    let replay = boundary_evidence()
        .lineage()
        .replay_derived_continuity(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(1),
        ))
        .with_provenance(
            boundary_evidence()
                .provenance()
                .replay_derived(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                    BoundaryArtifactLocator::new(BoundaryArtifactId::new(1), BoundaryArtifactField::Basis),
                ))
                .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
                .unwrap_success(),
        )
        .unwrap_success();

    let _ = replay.executed_receipt();
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
