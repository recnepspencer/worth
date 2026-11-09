use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::{
    BoundaryArtifactId, BoundaryArtifactLocator, CanonicalEquivalenceBasis,
    FoundationalBoundaryEvidenceComparisonBasis, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceProvenanceArtifact,
    FoundationalBoundaryEvidenceProvenanceFrontDoor, FoundationalBoundaryEvidenceSourceBasis,
};
use worth_proof::TransitionOutcome;

pub(super) fn provenance(
    locator: BoundaryArtifactLocator,
    freshness: FoundationalBoundaryEvidenceFreshnessPosture,
) -> FoundationalBoundaryEvidenceProvenanceArtifact {
    match FoundationalBoundaryEvidenceProvenanceFrontDoor
        .current(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            locator,
        ))
        .comparison_basis(FoundationalBoundaryEvidenceComparisonBasis::comparison(
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .with_freshness(freshness)
    {
        TransitionOutcome::Success(provenance) => provenance,
        _ => unreachable!("current retained Query deltas have a legal Foundational provenance"),
    }
}

pub(super) fn descriptive_boundary_id(
    delta: &super::WorthQueryConsumerInvalidationDelta,
) -> Result<BoundaryArtifactId, IdentityDerivationDenial> {
    let semantic = delta.semantic_projection()?;
    Ok(crate::domain_installation::foundational_boundary_artifact_id(semantic.identity()))
}
