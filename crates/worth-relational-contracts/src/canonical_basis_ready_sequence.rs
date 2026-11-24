use worth_foundational::facade::{CanonicalBasisReadyArtifact, CanonicalBasisSequence};

pub fn canonical_basis_ready_sequence(
    ready: &CanonicalBasisReadyArtifact,
) -> &CanonicalBasisSequence {
    ready.payload()
}
