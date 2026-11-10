use super::InstalledIdentityEvolutionKind;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::{
    admit_current_basis_boundary_evidence_attachment_bundle, boundary_evidence,
    foundational_boundary_evidence_attachment_readmission_authority, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    CurrentBasisBoundaryEvidenceAttachmentBundle,
    FoundationalBoundaryEvidenceAttestedLineageArtifact,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLineageSubject,
    FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidencePartialLineageArtifact,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalMaterializedBoundaryEvidenceAttachmentBundle,
};

#[derive(Clone, Debug)]
pub(super) struct FoundationalInstalledLineageIdentities {
    subject: BoundaryHandle,
    artifact: BoundaryArtifactId,
    subject_evidence_identity: WorthQueryEvidenceIdentity,
    source_basis_evidence_identity: WorthQueryEvidenceIdentity,
    receipt_evidence_identity: WorthQueryEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryFoundationalLineageAttachment {
    Attested {
        artifact: FoundationalBoundaryEvidenceAttestedLineageArtifact,
        materialized: FoundationalMaterializedBoundaryEvidenceAttachmentBundle,
        subject_evidence_identity: WorthQueryEvidenceIdentity,
        source_basis_evidence_identity: WorthQueryEvidenceIdentity,
        receipt_evidence_identity: WorthQueryEvidenceIdentity,
    },
    Partial {
        artifact: FoundationalBoundaryEvidencePartialLineageArtifact,
        materialized: FoundationalMaterializedBoundaryEvidenceAttachmentBundle,
        subject_evidence_identity: WorthQueryEvidenceIdentity,
        source_basis_evidence_identity: WorthQueryEvidenceIdentity,
        receipt_evidence_identity: WorthQueryEvidenceIdentity,
    },
}

impl WorthQueryFoundationalLineageAttachment {
    pub fn outcome_kind(
        &self,
    ) -> worth_foundational::facade::FoundationalBoundaryEvidenceLineageOutcomeKind {
        match self {
            Self::Attested { artifact, .. } => artifact.outcome_kind(),
            Self::Partial { artifact, .. } => artifact.outcome_kind(),
        }
    }

    pub fn subject_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::Attested {
                subject_evidence_identity,
                ..
            }
            | Self::Partial {
                subject_evidence_identity,
                ..
            } => subject_evidence_identity,
        }
    }

    pub fn source_basis_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::Attested {
                source_basis_evidence_identity,
                ..
            }
            | Self::Partial {
                source_basis_evidence_identity,
                ..
            } => source_basis_evidence_identity,
        }
    }

    pub fn receipt_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::Attested {
                receipt_evidence_identity,
                ..
            }
            | Self::Partial {
                receipt_evidence_identity,
                ..
            } => receipt_evidence_identity,
        }
    }

    pub fn materialized(&self) -> &FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
        match self {
            Self::Attested { materialized, .. } | Self::Partial { materialized, .. } => {
                materialized
            }
        }
    }

    pub fn admit_current_basis(&self) -> CurrentBasisBoundaryEvidenceAttachmentBundle {
        admit_current_basis_boundary_evidence_attachment_bundle(
            self.materialized().clone(),
            foundational_boundary_evidence_attachment_readmission_authority(),
        )
    }
}

impl FoundationalInstalledLineageIdentities {
    pub(super) fn from_execution(
        semantic: &WorthQueryEvidenceIdentity,
        basis_digest: &str,
        effect_receipt_identity: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let source_basis_evidence_identity =
            boundary_identity("installed-lineage-source-v1", basis_digest)?;
        let receipt_evidence_identity =
            boundary_identity("installed-lineage-receipt-v1", effect_receipt_identity)?;
        Ok(Self {
            subject: crate::domain_installation::foundational_boundary_handle(semantic),
            artifact: crate::domain_installation::foundational_boundary_artifact_id(
                &receipt_evidence_identity,
            ),
            subject_evidence_identity: semantic.clone(),
            source_basis_evidence_identity,
            receipt_evidence_identity,
        })
    }

    pub(super) fn attested_lineage(
        &self,
        kind: InstalledIdentityEvolutionKind,
        authoritative: bool,
    ) -> Option<WorthQueryFoundationalLineageAttachment> {
        if !authoritative
            && !matches!(
                kind,
                InstalledIdentityEvolutionKind::RetiredIdentity
                    | InstalledIdentityEvolutionKind::AdvisoryCorrespondence
                    | InstalledIdentityEvolutionKind::AmbiguousCorrespondence
                    | InstalledIdentityEvolutionKind::ContinuityBreak
            )
        {
            return None;
        }
        let source = BoundaryArtifactLocator::new(self.artifact, BoundaryArtifactField::Basis);
        let receipt_locator =
            BoundaryArtifactLocator::new(self.artifact, BoundaryArtifactField::Proofs);
        let provenance = boundary_evidence()
            .provenance()
            .current(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                source,
            ))
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
            .into_result()
            .ok()?;
        let subject = FoundationalBoundaryEvidenceLineageSubject::new(self.subject);
        let lineage = boundary_evidence().lineage();
        let receipt = boundary_evidence()
            .receipt()
            .execution(
                FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(receipt_locator),
            )
            .with_provenance(provenance.clone());
        let partial = match kind {
            InstalledIdentityEvolutionKind::AdvisoryCorrespondence => Some(
                lineage
                    .advisory_correspondence_candidate(subject)
                    .with_provenance(provenance.clone()),
            ),
            InstalledIdentityEvolutionKind::AmbiguousCorrespondence => Some(
                lineage
                    .ambiguity(subject)
                    .with_provenance(provenance.clone()),
            ),
            _ => None,
        };
        if let Some(artifact) = partial {
            return Some(self.partial_attachment(
                artifact,
                provenance,
                receipt.completed_receipt().clone(),
                receipt_locator,
            ));
        }
        let artifact = match kind {
            InstalledIdentityEvolutionKind::SplitSuccessors => lineage
                .plural_successor_predecessor(subject)
                .attested_by(receipt.clone()),
            InstalledIdentityEvolutionKind::MergedPredecessors => lineage
                .merge_successor(subject)
                .attested_by(receipt.clone()),
            InstalledIdentityEvolutionKind::RetiredIdentity
            | InstalledIdentityEvolutionKind::ContinuityBreak => {
                lineage.identity_break(subject).attested_by(receipt.clone())
            }
            InstalledIdentityEvolutionKind::PreservedIdentity
            | InstalledIdentityEvolutionKind::SingularSuccessor
            | InstalledIdentityEvolutionKind::GeneratedIdentity => {
                lineage.continuity(subject).attested_by(receipt.clone())
            }
            InstalledIdentityEvolutionKind::AdvisoryCorrespondence
            | InstalledIdentityEvolutionKind::AmbiguousCorrespondence
            | InstalledIdentityEvolutionKind::Denied => return None,
        };
        let materialized = boundary_evidence()
            .attachment()
            .for_boundary_artifact(receipt_locator)
            .with_attested_continuity(artifact.clone())
            .with_provenance_attachment(provenance)
            .with_receipt_attachment(receipt.completed_receipt().clone())
            .materialize_under(
                FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
            );
        Some(WorthQueryFoundationalLineageAttachment::Attested {
            artifact,
            materialized,
            subject_evidence_identity: self.subject_evidence_identity.clone(),
            source_basis_evidence_identity: self.source_basis_evidence_identity.clone(),
            receipt_evidence_identity: self.receipt_evidence_identity.clone(),
        })
    }

    fn partial_attachment(
        &self,
        artifact: FoundationalBoundaryEvidencePartialLineageArtifact,
        provenance: worth_foundational::facade::FoundationalBoundaryEvidenceProvenanceArtifact,
        receipt: worth_foundational::facade::FoundationalBoundaryEvidenceCompletedReceiptArtifact,
        target: BoundaryArtifactLocator,
    ) -> WorthQueryFoundationalLineageAttachment {
        let materialized = boundary_evidence()
            .attachment()
            .for_boundary_artifact(target)
            .with_partial_continuity(artifact.clone())
            .with_provenance_attachment(provenance)
            .with_receipt_attachment(receipt)
            .materialize_under(
                FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
            );
        WorthQueryFoundationalLineageAttachment::Partial {
            artifact,
            materialized,
            subject_evidence_identity: self.subject_evidence_identity.clone(),
            source_basis_evidence_identity: self.source_basis_evidence_identity.clone(),
            receipt_evidence_identity: self.receipt_evidence_identity.clone(),
        }
    }
}

fn boundary_identity(
    role: &'static str,
    value: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::InstalledDomainExecution,
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
        role,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("source_value"),
        value,
    )
    .seal()
}
