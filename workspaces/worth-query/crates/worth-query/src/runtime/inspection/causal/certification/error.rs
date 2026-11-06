use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::identity::CausalInspectionCertificationErrorIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionCertificationErrorKind {
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
    failure_identity: CausalInspectionCertificationErrorIdentity,
}

impl CausalInspectionCertificationError {
    pub(super) fn new(
        kind: CausalInspectionCertificationErrorKind,
        message: &'static str,
        evidence: &[String],
    ) -> Result<Self, IdentityDerivationDenial> {
        let failure_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::CausalInspectionCertificationError,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_value(WorthQueryEvidenceTag::new("message"), message)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("evidence"),
            evidence.iter().map(String::as_str),
        )
        .seal()?
        .into();
        Ok(Self {
            kind,
            message,
            failure_identity,
        })
    }

    pub fn kind(&self) -> CausalInspectionCertificationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }
}
