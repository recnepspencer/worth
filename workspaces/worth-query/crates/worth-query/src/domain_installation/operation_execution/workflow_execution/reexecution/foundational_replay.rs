use crate::domain_installation::{foundational_boundary_artifact_id, foundational_boundary_handle};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use worth_foundational::facade::{
    admit_current_basis_boundary_evidence_attachment_bundle, boundary_evidence,
    foundational_boundary_evidence_attachment_readmission_authority, BoundaryArtifactField,
    BoundaryArtifactLocator, CurrentBasisBoundaryEvidenceAttachmentBundle,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLineageSubject,
    FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalMaterializedBoundaryEvidenceAttachmentBundle,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorthQueryFoundationalReplayAttachment {
    original_trace_evidence_identity: WorthQueryEvidenceIdentity,
    replay_trace_evidence_identity: WorthQueryEvidenceIdentity,
    materialized: FoundationalMaterializedBoundaryEvidenceAttachmentBundle,
}

impl WorthQueryFoundationalReplayAttachment {
    pub fn original_trace_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.original_trace_evidence_identity
    }

    pub fn replay_trace_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.replay_trace_evidence_identity
    }

    pub fn materialized(&self) -> &FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
        &self.materialized
    }

    pub fn admit_current_basis(&self) -> CurrentBasisBoundaryEvidenceAttachmentBundle {
        admit_current_basis_boundary_evidence_attachment_bundle(
            self.materialized.clone(),
            foundational_boundary_evidence_attachment_readmission_authority(),
        )
    }
}

pub(crate) fn materialize_replay_attachment(
    original_trace_identity: &str,
    replay_trace_identity: &str,
    equivalent: bool,
) -> WorthQueryFoundationalReplayAttachment {
    let original = trace_evidence_identity("replay-original-trace-v1", original_trace_identity);
    let replay = trace_evidence_identity("replay-candidate-trace-v1", replay_trace_identity);
    let source = BoundaryArtifactLocator::new(
        foundational_boundary_artifact_id(&original),
        BoundaryArtifactField::Basis,
    );
    let target = BoundaryArtifactLocator::new(
        foundational_boundary_artifact_id(&replay),
        BoundaryArtifactField::Proofs,
    );
    let provenance = boundary_evidence()
        .provenance()
        .replay_derived(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            source,
        ))
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .into_result()
        .expect("fixed replay-derived Foundational provenance is legal");
    let receipt = boundary_evidence()
        .receipt()
        .execution(FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(target))
        .with_provenance(provenance.clone());
    let mut attachment = boundary_evidence()
        .attachment()
        .for_boundary_artifact(target)
        .with_provenance_attachment(provenance.clone())
        .with_receipt_attachment(receipt.completed_receipt().clone());
    if equivalent {
        let lineage = boundary_evidence()
            .lineage()
            .replay_derived_continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                foundational_boundary_handle(&original),
            ))
            .with_provenance(provenance)
            .into_result()
            .expect("fixed replay-derived Foundational lineage is legal");
        attachment = attachment.with_replay_derived_continuity(lineage);
    }
    WorthQueryFoundationalReplayAttachment {
        original_trace_evidence_identity: original,
        replay_trace_evidence_identity: replay,
        materialized: attachment.materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        ),
    }
}

fn trace_evidence_identity(
    role: &'static str,
    trace_identity: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::InstalledDomainExecution)
            .field_shape(WorthQueryEvidenceTag::new("identity_family"), role)
            .field_value(
                WorthQueryEvidenceTag::new("query_trace_identity"),
                trace_identity,
            )
            .seal()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::FoundationalBoundaryEvidenceContinuityAttachmentScope;

    #[test]
    fn divergence_materializes_receipt_and_provenance_without_continuity() {
        let divergent = materialize_replay_attachment("original", "candidate", false);
        let equivalent = materialize_replay_attachment("original", "candidate", true);

        assert_eq!(divergent.materialized().continuity_scope(), None);
        assert_eq!(
            equivalent.materialized().continuity_scope(),
            Some(FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel)
        );
        assert_eq!(
            divergent.materialized().target(),
            equivalent.materialized().target()
        );
    }
}
