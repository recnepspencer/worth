use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::identity::CausalInspectionCertificationFailureEvidenceIdentity;
use super::error::{CausalInspectionCertificationError, CausalInspectionCertificationErrorKind};
use super::matrix_kind::CausalInspectionRepresentativeKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionCertificationFailureKind {
    RelationalAuthorityMismatch,
    RedactionPolicyOverclaim,
    UnsupportedExplanationFamily,
    DirectBridgeDiagnosticsDomainExplanationForbidden,
    DirectRelationalRuntimeDomainExplanationForbidden,
    DirectSignalGraphDomainExplanationForbidden,
    DurableCausalArchiveOverclaimForbidden,
    StoreBackedReplayReconstructionOverclaimForbidden,
}

impl CausalInspectionCertificationFailureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RelationalAuthorityMismatch => "relational_authority_mismatch",
            Self::RedactionPolicyOverclaim => "redaction_policy_overclaim",
            Self::UnsupportedExplanationFamily => "unsupported_explanation_family",
            Self::DirectBridgeDiagnosticsDomainExplanationForbidden => {
                "direct_bridge_diagnostics_domain_explanation_forbidden"
            }
            Self::DirectRelationalRuntimeDomainExplanationForbidden => {
                "direct_relational_runtime_domain_explanation_forbidden"
            }
            Self::DirectSignalGraphDomainExplanationForbidden => {
                "direct_signal_graph_domain_explanation_forbidden"
            }
            Self::DurableCausalArchiveOverclaimForbidden => {
                "durable_causal_archive_overclaim_forbidden"
            }
            Self::StoreBackedReplayReconstructionOverclaimForbidden => {
                "store_backed_replay_reconstruction_overclaim_forbidden"
            }
        }
    }

    pub fn representative_kind(&self) -> CausalInspectionRepresentativeKind {
        match self {
            Self::RelationalAuthorityMismatch => {
                CausalInspectionRepresentativeKind::RelationalAuthorityMismatchDenied
            }
            Self::RedactionPolicyOverclaim => {
                CausalInspectionRepresentativeKind::RedactionPolicyOverclaimDenied
            }
            Self::UnsupportedExplanationFamily => {
                CausalInspectionRepresentativeKind::UnsupportedExplanationFamilyDenied
            }
            Self::DirectBridgeDiagnosticsDomainExplanationForbidden => {
                CausalInspectionRepresentativeKind::DirectBridgeDiagnosticsDomainExplanationForbidden
            }
            Self::DirectRelationalRuntimeDomainExplanationForbidden => {
                CausalInspectionRepresentativeKind::DirectRelationalRuntimeDomainExplanationForbidden
            }
            Self::DirectSignalGraphDomainExplanationForbidden => {
                CausalInspectionRepresentativeKind::DirectSignalGraphDomainExplanationForbidden
            }
            Self::DurableCausalArchiveOverclaimForbidden => {
                CausalInspectionRepresentativeKind::DurableCausalArchiveOverclaimForbidden
            }
            Self::StoreBackedReplayReconstructionOverclaimForbidden => {
                CausalInspectionRepresentativeKind::StoreBackedReplayReconstructionOverclaimForbidden
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionCertificationFailureSource {
    AdmissionBoundary,
    MaterializationPolicy,
    PublicBoundaryAudit,
    LaterMilestoneDebt,
}

impl CausalInspectionCertificationFailureSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdmissionBoundary => "admission_boundary",
            Self::MaterializationPolicy => "materialization_policy",
            Self::PublicBoundaryAudit => "public_boundary_audit",
            Self::LaterMilestoneDebt => "later_milestone_debt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionCertificationFailureEvidence {
    kind: CausalInspectionCertificationFailureKind,
    source: CausalInspectionCertificationFailureSource,
    ordinary_path_forbidden: bool,
    later_milestone_debt: bool,
    failure_identity: CausalInspectionCertificationFailureEvidenceIdentity,
}

impl CausalInspectionCertificationFailureEvidence {
    pub fn for_representative_kind(
        representative_kind: CausalInspectionRepresentativeKind,
    ) -> Result<Self, CausalInspectionCertificationError> {
        let Some(kind) = certification_failure_kind(representative_kind) else {
            return Err(CausalInspectionCertificationError::new(
                CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch,
                "representative kind is not a typed certification failure lane",
                &[format!("kind:{}", representative_kind.as_str())],
            ));
        };
        Ok(Self::from_kind(kind))
    }

    pub fn from_kind(
        kind: CausalInspectionCertificationFailureKind,
    ) -> Result<Self, IdentityDerivationDenial> {
        let (source, ordinary_path_forbidden, later_milestone_debt) = failure_posture(kind);
        let failure_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_shape(WorthQueryEvidenceTag::new("source"), source.as_str())
        .field_bool(
            WorthQueryEvidenceTag::new("ordinary_path_forbidden"),
            ordinary_path_forbidden,
        )
        .field_bool(
            WorthQueryEvidenceTag::new("later_milestone_debt"),
            later_milestone_debt,
        )
        .seal()?
        .into();
        Ok(Self {
            kind,
            source,
            ordinary_path_forbidden,
            later_milestone_debt,
            failure_identity,
        })
    }

    pub fn kind(&self) -> CausalInspectionCertificationFailureKind {
        self.kind
    }

    pub fn representative_kind(&self) -> CausalInspectionRepresentativeKind {
        self.kind.representative_kind()
    }

    pub fn source(&self) -> CausalInspectionCertificationFailureSource {
        self.source
    }

    pub fn ordinary_path_forbidden(&self) -> bool {
        self.ordinary_path_forbidden
    }

    pub fn later_milestone_debt(&self) -> bool {
        self.later_milestone_debt
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }
}

pub(super) fn certification_failure_kind(
    kind: CausalInspectionRepresentativeKind,
) -> Option<CausalInspectionCertificationFailureKind> {
    match kind {
        CausalInspectionRepresentativeKind::RelationalAuthorityMismatchDenied => {
            Some(CausalInspectionCertificationFailureKind::RelationalAuthorityMismatch)
        }
        CausalInspectionRepresentativeKind::RedactionPolicyOverclaimDenied => {
            Some(CausalInspectionCertificationFailureKind::RedactionPolicyOverclaim)
        }
        CausalInspectionRepresentativeKind::UnsupportedExplanationFamilyDenied => {
            Some(CausalInspectionCertificationFailureKind::UnsupportedExplanationFamily)
        }
        CausalInspectionRepresentativeKind::DirectBridgeDiagnosticsDomainExplanationForbidden => {
            Some(CausalInspectionCertificationFailureKind::DirectBridgeDiagnosticsDomainExplanationForbidden)
        }
        CausalInspectionRepresentativeKind::DirectRelationalRuntimeDomainExplanationForbidden => {
            Some(CausalInspectionCertificationFailureKind::DirectRelationalRuntimeDomainExplanationForbidden)
        }
        CausalInspectionRepresentativeKind::DirectSignalGraphDomainExplanationForbidden => {
            Some(CausalInspectionCertificationFailureKind::DirectSignalGraphDomainExplanationForbidden)
        }
        CausalInspectionRepresentativeKind::DurableCausalArchiveOverclaimForbidden => {
            Some(CausalInspectionCertificationFailureKind::DurableCausalArchiveOverclaimForbidden)
        }
        CausalInspectionRepresentativeKind::StoreBackedReplayReconstructionOverclaimForbidden => {
            Some(CausalInspectionCertificationFailureKind::StoreBackedReplayReconstructionOverclaimForbidden)
        }
        _ => None,
    }
}

fn failure_posture(
    kind: CausalInspectionCertificationFailureKind,
) -> (CausalInspectionCertificationFailureSource, bool, bool) {
    match kind {
        CausalInspectionCertificationFailureKind::RelationalAuthorityMismatch => {
            (CausalInspectionCertificationFailureSource::AdmissionBoundary, false, false)
        }
        CausalInspectionCertificationFailureKind::RedactionPolicyOverclaim => {
            (CausalInspectionCertificationFailureSource::MaterializationPolicy, false, false)
        }
        CausalInspectionCertificationFailureKind::UnsupportedExplanationFamily => {
            (CausalInspectionCertificationFailureSource::AdmissionBoundary, false, true)
        }
        CausalInspectionCertificationFailureKind::DirectBridgeDiagnosticsDomainExplanationForbidden
        | CausalInspectionCertificationFailureKind::DirectRelationalRuntimeDomainExplanationForbidden
        | CausalInspectionCertificationFailureKind::DirectSignalGraphDomainExplanationForbidden => {
            (CausalInspectionCertificationFailureSource::PublicBoundaryAudit, true, false)
        }
        CausalInspectionCertificationFailureKind::DurableCausalArchiveOverclaimForbidden
        | CausalInspectionCertificationFailureKind::StoreBackedReplayReconstructionOverclaimForbidden => {
            (CausalInspectionCertificationFailureSource::LaterMilestoneDebt, true, true)
        }
    }
}
