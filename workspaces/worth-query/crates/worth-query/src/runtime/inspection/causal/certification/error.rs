use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::identity::CausalInspectionCertificationErrorIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionCertificationErrorKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    MissingRequiredHostileLane,
    MissingRepresentativeMatrixRow,
    RepresentativeMatrixMismatch,
    RedactionIdentityDrift,
    PublicBoundaryBypass,
    ScaleSlopeDrift,
    ProofShapeBypass,
}

impl CausalInspectionCertificationErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::MissingRequiredHostileLane => "missing_required_hostile_lane",
            Self::MissingRepresentativeMatrixRow => "missing_representative_matrix_row",
            Self::RepresentativeMatrixMismatch => "representative_matrix_mismatch",
            Self::RedactionIdentityDrift => "redaction_identity_drift",
            Self::PublicBoundaryBypass => "public_boundary_bypass",
            Self::ScaleSlopeDrift => "scale_slope_drift",
            Self::ProofShapeBypass => "proof_shape_bypass",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionCertificationError {
    kind: CausalInspectionCertificationErrorKind,
    message: &'static str,
    failure_identity: Result<CausalInspectionCertificationErrorIdentity, IdentityDerivationDenial>,
}

impl CausalInspectionCertificationError {
    pub(super) fn new(
        kind: CausalInspectionCertificationErrorKind,
        message: &'static str,
        evidence: &[String],
    ) -> Self {
        let failure_identity = (|| {
            Ok(worth_query_evidence_identity(
                WorthQueryEvidenceScope::CausalInspectionCertificationError,
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_value(WorthQueryEvidenceTag::new("message"), message)
            .field_value_sequence(
                WorthQueryEvidenceTag::new("evidence"),
                evidence.iter().map(String::as_str),
            )
            .seal()?
            .into())
        })();
        Self {
            kind,
            message,
            failure_identity,
        }
    }

    pub fn kind(&self) -> CausalInspectionCertificationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_digest(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self
            .failure_identity
            .as_ref()
            .map_err(|cause| *cause)?
            .as_str())
    }
}

impl From<IdentityDerivationDenial> for CausalInspectionCertificationError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            kind: CausalInspectionCertificationErrorKind::EvidenceIdentityDerivationDenied(cause),
            message: "causal inspection certification identity refused",
            failure_identity: Err(cause),
        }
    }
}
