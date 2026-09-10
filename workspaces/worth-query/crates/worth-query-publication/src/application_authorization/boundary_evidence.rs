use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    boundary_evidence, boundary_receipt_category_of, BoundaryArtifactField, BoundaryArtifactId,
    BoundaryArtifactLocator, BoundaryProfiledArtifact, FoundationalBoundaryArtifactCategory,
    FoundationalBoundaryCategoryConstructionDenial,
    FoundationalBoundaryEvidenceExecutedReceiptArtifact,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceProvenanceArtifact,
    FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryReceiptSurface, FoundationalDiagnosticExplanationBundle,
    FoundationalDiagnosticMaterializationDenial, FoundationalProfileProgressionDenial,
};
use worth_proof::TransitionOutcome;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitPublicationSource, WorthQueryApprovedElevation,
    WorthQueryElevationClosureKind, WorthQueryMandatoryReview, WorthQueryRequestedElevation,
    WorthQueryReviewedElevation,
};

use crate::application_aftermath::WorthQueryPublishedApplicationCommitBoundaryEvidence;

use super::explanation::{
    materialize_explanation, WorthQueryPublishedApplicationAuthorizationKind,
};

pub(super) mod profile;
#[cfg(test)]
mod tests;

pub use profile::{
    WorthQueryApplicationAuthorizationProfileStage,
    WorthQueryApplicationAuthorizationPublicationProfile,
};

#[derive(Clone, Copy)]
pub(super) struct WorthQueryApplicationAuthorizationBoundaryIdentity {
    locator: BoundaryArtifactLocator,
}

impl WorthQueryApplicationAuthorizationBoundaryIdentity {
    pub(super) fn from_closed_publication(
        family: &'static str,
        axes: &[String],
        profile: WorthQueryApplicationAuthorizationPublicationProfile,
    ) -> Self {
        let mut digest = Sha256::new();
        append_identity_axis(&mut digest, b"worth.query.publication.authorization.v1");
        append_identity_axis(&mut digest, family.as_bytes());
        for axis in axes {
            append_identity_axis(&mut digest, axis.as_bytes());
        }
        append_profile(&mut digest, profile);
        let digest = digest.finalize();
        let mut prefix = [0_u8; 8];
        prefix.copy_from_slice(&digest[..8]);
        let identity = u64::from_be_bytes(prefix).max(1);
        Self {
            locator: BoundaryArtifactLocator::new(
                BoundaryArtifactId::new(identity),
                BoundaryArtifactField::Payload,
            ),
        }
    }

    pub(super) const fn artifact_id(self) -> BoundaryArtifactId {
        self.locator.artifact_id()
    }

    pub(super) const fn locator(self) -> BoundaryArtifactLocator {
        self.locator
    }
}

fn append_identity_axis(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn append_profile(
    digest: &mut Sha256,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) {
    for profile in [
        profile.requested(),
        profile.admitted(),
        profile.materialized(),
    ] {
        append_identity_axis(digest, profile_axis(profile).as_bytes());
    }
}

fn profile_axis(profile: worth_foundational::facade::FoundationalProfileSet) -> String {
    use worth_foundational::facade::{
        AdmissionReadinessProfile as Admission, CertificationPostureProfile as Certification,
        CompatibilityPostureProfile as Compatibility, DiagnosticRichnessProfile as Diagnostic,
        RetentionDeliveryProfile as Retention, SupportPostureProfile as Support,
    };
    let diagnostic = match profile.diagnostic_richness() {
        Diagnostic::OperationalMinimal => "operational-minimal",
        Diagnostic::Standard => "standard",
        Diagnostic::Forensic => "forensic",
    };
    let support = match profile.support_posture() {
        Support::InternalOnly => "internal-only",
        Support::SupportReady => "support-ready",
        Support::CertificationReady => "certification-ready",
    };
    let compatibility = match profile.compatibility_posture() {
        Compatibility::NativeOnly => "native-only",
        Compatibility::CompatibilityLowered => "compatibility-lowered",
        Compatibility::CompatibilityRequired => "compatibility-required",
    };
    let admission = match profile.admission_readiness() {
        Admission::CandidateOnly => "candidate-only",
        Admission::Admitted => "admitted",
        Admission::ProductionGateReady => "production-gate-ready",
    };
    let retention = match profile.retention_delivery() {
        Retention::Ephemeral => "ephemeral",
        Retention::Retained => "retained",
        Retention::Durable => "durable",
    };
    let certification = match profile.certification_posture() {
        Certification::Uncertified => "uncertified",
        Certification::EvidenceBacked => "evidence-backed",
        Certification::ProductionCertified => "production-certified",
    };
    format!("{diagnostic}/{support}/{compatibility}/{admission}/{retention}/{certification}")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationAuthorizationPublicationDenial {
    OutcomeNotPublishable,
    BoundaryCategory(FoundationalBoundaryCategoryConstructionDenial),
    ProfileAdmission(FoundationalProfileProgressionDenial),
    ProfileMaterialization(FoundationalProfileProgressionDenial),
    Diagnostic(FoundationalDiagnosticMaterializationDenial),
    Provenance(FoundationalBoundaryEvidenceProvenanceConstructionDenial),
}

/// Descriptive publication output. It is not an approved elevation and cannot
/// be substituted for Query's active-use authority.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;
/// use worth_query_publication::facade::domain_computation::
///     WorthQueryPublishedApplicationAuthorization;
///
/// fn requires_query_authority(_: &WorthQueryApprovedElevation) {}
///
/// fn published_description_cannot_authorize(
///     published: &WorthQueryPublishedApplicationAuthorization,
/// ) {
///     requires_query_authority(published);
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryPublishedApplicationAuthorization {
    kind: WorthQueryPublishedApplicationAuthorizationKind,
    query_boundary_evidence: WorthQueryPublishedApplicationCommitBoundaryEvidence,
    boundary: BoundaryProfiledArtifact<FoundationalBoundaryReceiptSurface>,
    explanation: FoundationalDiagnosticExplanationBundle,
    provenance: FoundationalBoundaryEvidenceProvenanceArtifact,
    publication_receipt: FoundationalBoundaryEvidenceExecutedReceiptArtifact,
}

impl WorthQueryPublishedApplicationAuthorization {
    pub const fn kind(&self) -> WorthQueryPublishedApplicationAuthorizationKind {
        self.kind
    }

    pub const fn query_boundary_evidence(
        &self,
    ) -> &WorthQueryPublishedApplicationCommitBoundaryEvidence {
        &self.query_boundary_evidence
    }

    pub const fn boundary(&self) -> &BoundaryProfiledArtifact<FoundationalBoundaryReceiptSurface> {
        &self.boundary
    }

    pub fn boundary_category(&self) -> FoundationalBoundaryArtifactCategory {
        boundary_receipt_category_of(self.boundary.payload().payload())
    }

    pub const fn explanation(&self) -> &FoundationalDiagnosticExplanationBundle {
        &self.explanation
    }

    pub const fn provenance(&self) -> &FoundationalBoundaryEvidenceProvenanceArtifact {
        &self.provenance
    }

    pub const fn publication_receipt(
        &self,
    ) -> &FoundationalBoundaryEvidenceExecutedReceiptArtifact {
        &self.publication_receipt
    }
}

pub fn publish_requested_elevation(
    requested: &WorthQueryRequestedElevation,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryPublishedApplicationAuthorization,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let source = requested.publication_source();
    publish_transition(
        WorthQueryPublishedApplicationAuthorizationKind::ElevationRequested,
        &source,
        profile,
    )
}

pub fn publish_approved_elevation(
    approved: &WorthQueryApprovedElevation,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryPublishedApplicationAuthorization,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let source = approved.publication_source();
    publish_transition(
        WorthQueryPublishedApplicationAuthorizationKind::ElevationApproved,
        &source,
        profile,
    )
}

pub fn publish_mandatory_review(
    review: &WorthQueryMandatoryReview,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryPublishedApplicationAuthorization,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let kind = match review.closure_kind() {
        WorthQueryElevationClosureKind::Revoked => {
            WorthQueryPublishedApplicationAuthorizationKind::RevokedReviewRequired
        }
        WorthQueryElevationClosureKind::Expired => {
            WorthQueryPublishedApplicationAuthorizationKind::ExpiredReviewRequired
        }
    };
    let source = review.publication_source();
    publish_transition(kind, &source, profile)
}

pub fn publish_reviewed_elevation(
    reviewed: &WorthQueryReviewedElevation,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryPublishedApplicationAuthorization,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let kind = match reviewed.closure_kind() {
        WorthQueryElevationClosureKind::Revoked => {
            WorthQueryPublishedApplicationAuthorizationKind::RevokedElevationReviewed
        }
        WorthQueryElevationClosureKind::Expired => {
            WorthQueryPublishedApplicationAuthorizationKind::ExpiredElevationReviewed
        }
    };
    let source = reviewed.publication_source();
    publish_transition(kind, &source, profile)
}

fn publish_transition(
    kind: WorthQueryPublishedApplicationAuthorizationKind,
    source: &WorthQueryApplicationCommitPublicationSource,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryPublishedApplicationAuthorization,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let identity = WorthQueryApplicationAuthorizationBoundaryIdentity::from_closed_publication(
        kind.diagnostic_code(),
        &[source.emitted_effect_count().to_string()],
        profile,
    );
    let lowered = lower_boundary_material(kind, identity, source.emitted_effect_count(), profile)?;
    Ok(WorthQueryPublishedApplicationAuthorization {
        kind,
        query_boundary_evidence:
            WorthQueryPublishedApplicationCommitBoundaryEvidence::from_publication_source(source),
        boundary: lowered.boundary,
        explanation: lowered.explanation,
        provenance: lowered.provenance,
        publication_receipt: lowered.publication_receipt,
    })
}

struct WorthQueryLoweredApplicationAuthorizationPublication {
    boundary: BoundaryProfiledArtifact<FoundationalBoundaryReceiptSurface>,
    explanation: FoundationalDiagnosticExplanationBundle,
    provenance: FoundationalBoundaryEvidenceProvenanceArtifact,
    publication_receipt: FoundationalBoundaryEvidenceExecutedReceiptArtifact,
}

fn lower_boundary_material(
    kind: WorthQueryPublishedApplicationAuthorizationKind,
    identity: WorthQueryApplicationAuthorizationBoundaryIdentity,
    attested_effect_count: usize,
    profile: WorthQueryApplicationAuthorizationPublicationProfile,
) -> Result<
    WorthQueryLoweredApplicationAuthorizationPublication,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    let boundary = profile::profile_boundary(kind, attested_effect_count, profile)?;
    let explanation = materialize_explanation(kind, identity, profile.materialized())
        .map_err(WorthQueryApplicationAuthorizationPublicationDenial::Diagnostic)?;
    let provenance = current_provenance(identity)?;
    let publication_receipt = boundary_evidence()
        .receipt()
        .publication(
            FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(identity.locator()),
        )
        .with_provenance(provenance.clone());
    Ok(WorthQueryLoweredApplicationAuthorizationPublication {
        boundary,
        explanation,
        provenance,
        publication_receipt,
    })
}

pub(super) fn current_provenance(
    identity: WorthQueryApplicationAuthorizationBoundaryIdentity,
) -> Result<
    FoundationalBoundaryEvidenceProvenanceArtifact,
    WorthQueryApplicationAuthorizationPublicationDenial,
> {
    match boundary_evidence()
        .provenance()
        .current(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            identity.locator(),
        ))
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
    {
        TransitionOutcome::Success(provenance) => Ok(provenance),
        TransitionOutcome::Denied(denial) => {
            Err(WorthQueryApplicationAuthorizationPublicationDenial::Provenance(denial))
        }
        _ => unreachable!("Foundational provenance construction has no nonterminal outcome"),
    }
}
