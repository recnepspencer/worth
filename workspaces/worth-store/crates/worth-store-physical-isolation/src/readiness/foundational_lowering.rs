#[cfg(feature = "certification-authority")]
use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    FoundationalBoundaryEvidenceReceiptBoundary,
};
use worth_foundational::{
    FoundationalBoundaryEvidenceExecutedReceiptArtifact,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceProvenanceArtifact,
    FoundationalBoundaryEvidenceSourceBasis,
};

#[cfg(feature = "certification-authority")]
use super::PhysicalIsolationEntryIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationEntryFoundationalEvidence {
    executed_receipt: FoundationalBoundaryEvidenceExecutedReceiptArtifact,
    provenance: FoundationalBoundaryEvidenceProvenanceArtifact,
    source_basis: FoundationalBoundaryEvidenceSourceBasis,
    freshness_posture: FoundationalBoundaryEvidenceFreshnessPosture,
}

impl PhysicalIsolationEntryFoundationalEvidence {
    #[cfg(feature = "certification-authority")]
    pub(crate) fn lower(
        identity: &PhysicalIsolationEntryIdentity,
    ) -> Result<Self, FoundationalBoundaryEvidenceProvenanceConstructionDenial> {
        let source_basis = source_basis(identity);
        let freshness_posture =
            FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay;
        let provenance = boundary_evidence()
            .provenance()
            .replay_derived(source_basis.clone())
            .with_freshness(freshness_posture)
            .into_result()?;
        let executed_receipt = boundary_evidence()
            .receipt()
            .execution(receipt_boundary(identity))
            .with_provenance(provenance.clone());
        Ok(Self {
            executed_receipt,
            provenance,
            source_basis,
            freshness_posture,
        })
    }

    pub const fn executed_receipt(&self) -> &FoundationalBoundaryEvidenceExecutedReceiptArtifact {
        &self.executed_receipt
    }

    pub const fn provenance(&self) -> &FoundationalBoundaryEvidenceProvenanceArtifact {
        &self.provenance
    }

    pub const fn source_basis(&self) -> &FoundationalBoundaryEvidenceSourceBasis {
        &self.source_basis
    }

    pub const fn freshness_posture(&self) -> FoundationalBoundaryEvidenceFreshnessPosture {
        self.freshness_posture
    }

    pub const fn is_store_physical_stability_authority(&self) -> bool {
        false
    }
}

#[cfg(feature = "certification-authority")]
fn source_basis(
    identity: &PhysicalIsolationEntryIdentity,
) -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(identity.boundary_artifact_id()),
        BoundaryArtifactField::Basis,
    ))
}

#[cfg(feature = "certification-authority")]
fn receipt_boundary(
    identity: &PhysicalIsolationEntryIdentity,
) -> FoundationalBoundaryEvidenceReceiptBoundary {
    FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(identity.boundary_artifact_id()),
        BoundaryArtifactField::Payload,
    ))
}
