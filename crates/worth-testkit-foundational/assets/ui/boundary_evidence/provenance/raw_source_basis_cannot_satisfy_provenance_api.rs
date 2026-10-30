use worth_foundational::{
    BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalBoundaryEvidenceProvenanceArtifact, FoundationalBoundaryEvidenceSourceBasis,
};

fn needs_provenance(_provenance: FoundationalBoundaryEvidenceProvenanceArtifact) {}

fn main() {
    let source_basis = FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(1), BoundaryArtifactField::Basis),
    );

    needs_provenance(source_basis);
}
