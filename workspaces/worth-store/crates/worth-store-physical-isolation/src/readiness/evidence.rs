#[cfg(feature = "certification-authority")]
use super::PhysicalIsolationEntryIdentity;
use super::{
    foundational_lowering::PhysicalIsolationEntryFoundationalEvidence,
    proof_progression::PhysicalIsolationEntryProofProgression,
};

#[derive(Debug, Clone)]
pub struct PhysicalIsolationEntryEvidence {
    foundational: PhysicalIsolationEntryFoundationalEvidence,
    proof_progression: PhysicalIsolationEntryProofProgression,
}

impl PhysicalIsolationEntryEvidence {
    #[cfg(feature = "certification-authority")]
    pub fn from_entry_identity(
        identity: &PhysicalIsolationEntryIdentity,
    ) -> Result<Self, worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial>
    {
        Ok(Self {
            foundational: PhysicalIsolationEntryFoundationalEvidence::lower(identity)?,
            proof_progression: PhysicalIsolationEntryProofProgression::from_identity(
                identity.clone(),
            ),
        })
    }

    pub const fn foundational(&self) -> &PhysicalIsolationEntryFoundationalEvidence {
        &self.foundational
    }

    pub const fn proof_progression(&self) -> &PhysicalIsolationEntryProofProgression {
        &self.proof_progression
    }

    pub const fn is_store_physical_stability_authority(&self) -> bool {
        false
    }
}
