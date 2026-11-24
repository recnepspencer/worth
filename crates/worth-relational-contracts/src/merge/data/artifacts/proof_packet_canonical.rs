use worth_foundational::{CanonicalBasisEntry, CanonicalBasisReadyArtifact};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMergeProofPacketCanonicalBasis {
    artifact: CanonicalBasisReadyArtifact,
}

impl RelationalMergeProofPacketCanonicalBasis {
    pub fn artifact(&self) -> &CanonicalBasisReadyArtifact {
        &self.artifact
    }

    pub fn entries(&self) -> &[CanonicalBasisEntry] {
        self.artifact.payload().entries()
    }
}

/// Operations used by the Relational owners.
pub trait RelationalMergeProofPacketCanonicalBasisOwnerPort: Sized {
    fn new(artifact: CanonicalBasisReadyArtifact) -> Self;
}

impl RelationalMergeProofPacketCanonicalBasisOwnerPort
    for RelationalMergeProofPacketCanonicalBasis
{
    fn new(artifact: CanonicalBasisReadyArtifact) -> Self {
        Self { artifact }
    }
}
