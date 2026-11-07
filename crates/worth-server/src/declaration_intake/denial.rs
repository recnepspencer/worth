use worth_foundational::DiagnosticRichnessProfile;

use super::WorthServerDirectSupportSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerDirectDeclarationDenial {
    EvidenceIdentityDerivationDenied {
        diagnostics_profile: DiagnosticRichnessProfile,
        cause: worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
        detail: String,
    },
    InvalidDeclarationIdentity {
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: String,
    },
    WorkspaceBindingFailed {
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: String,
    },
    SourceNotAdmitted {
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: String,
        support_snapshot: WorthServerDirectSupportSnapshot,
    },
    QueryFacadeFamilyNotAdmitted {
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: String,
        support_snapshot: WorthServerDirectSupportSnapshot,
    },
}

impl WorthServerDirectDeclarationDenial {
    pub(crate) fn invalid_declaration_identity(
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
    ) -> Self {
        Self::InvalidDeclarationIdentity {
            diagnostics_profile,
            detail: detail.into(),
        }
    }

    pub(crate) fn workspace_binding_failed(
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
    ) -> Self {
        Self::WorkspaceBindingFailed {
            diagnostics_profile,
            detail: detail.into(),
        }
    }

    pub(crate) fn source_not_admitted(
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
        support_snapshot: WorthServerDirectSupportSnapshot,
    ) -> Self {
        Self::SourceNotAdmitted {
            diagnostics_profile,
            detail: detail.into(),
            support_snapshot,
        }
    }

    pub(crate) fn query_facade_family_not_admitted(
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
        support_snapshot: WorthServerDirectSupportSnapshot,
    ) -> Self {
        Self::QueryFacadeFamilyNotAdmitted {
            diagnostics_profile,
            detail: detail.into(),
            support_snapshot,
        }
    }

    pub fn code(&self) -> WorthServerDirectDeclarationDenialCode {
        match self {
            Self::EvidenceIdentityDerivationDenied { cause, .. } => {
                WorthServerDirectDeclarationDenialCode::EvidenceIdentityDerivationDenied(*cause)
            }
            Self::InvalidDeclarationIdentity { .. } => {
                WorthServerDirectDeclarationDenialCode::InvalidDeclarationIdentity
            }
            Self::WorkspaceBindingFailed { .. } => {
                WorthServerDirectDeclarationDenialCode::WorkspaceBindingFailed
            }
            Self::SourceNotAdmitted { .. } => {
                WorthServerDirectDeclarationDenialCode::SourceNotAdmitted
            }
            Self::QueryFacadeFamilyNotAdmitted { .. } => {
                WorthServerDirectDeclarationDenialCode::QueryFacadeFamilyNotAdmitted
            }
        }
    }

    pub fn diagnostics_profile(&self) -> DiagnosticRichnessProfile {
        match self {
            Self::EvidenceIdentityDerivationDenied {
                diagnostics_profile,
                ..
            }
            | Self::InvalidDeclarationIdentity {
                diagnostics_profile,
                ..
            }
            | Self::WorkspaceBindingFailed {
                diagnostics_profile,
                ..
            }
            | Self::SourceNotAdmitted {
                diagnostics_profile,
                ..
            }
            | Self::QueryFacadeFamilyNotAdmitted {
                diagnostics_profile,
                ..
            } => *diagnostics_profile,
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::EvidenceIdentityDerivationDenied { detail, .. }
            | Self::InvalidDeclarationIdentity { detail, .. }
            | Self::WorkspaceBindingFailed { detail, .. }
            | Self::SourceNotAdmitted { detail, .. }
            | Self::QueryFacadeFamilyNotAdmitted { detail, .. } => detail,
        }
    }

    pub fn support_snapshot(&self) -> Option<&WorthServerDirectSupportSnapshot> {
        match self {
            Self::SourceNotAdmitted {
                support_snapshot, ..
            }
            | Self::QueryFacadeFamilyNotAdmitted {
                support_snapshot, ..
            } => Some(support_snapshot),
            Self::EvidenceIdentityDerivationDenied { .. }
            | Self::InvalidDeclarationIdentity { .. }
            | Self::WorkspaceBindingFailed { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerDirectDeclarationDenialCode {
    EvidenceIdentityDerivationDenied(
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    InvalidDeclarationIdentity,
    WorkspaceBindingFailed,
    SourceNotAdmitted,
    QueryFacadeFamilyNotAdmitted,
}
