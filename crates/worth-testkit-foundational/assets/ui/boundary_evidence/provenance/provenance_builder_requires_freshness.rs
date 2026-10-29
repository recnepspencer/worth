use worth_foundational::{
    boundary_evidence_api::common_path, BoundaryArtifactField, BoundaryArtifactId,
    BoundaryArtifactLocator, FoundationalBoundaryEvidenceSourceBasis,
};

fn main() {
    let source_basis = FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(1), BoundaryArtifactField::Basis),
    );

    let _ = common_path::provenance().current(source_basis).finish();
}
