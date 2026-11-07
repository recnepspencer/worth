#[cfg(test)]
use super::cost::HistoricalPathCostPosture;
use super::path_classes::{
    AdmittedHistoricalPathClass, RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoricalEvaluationFailureClass {
    EvidenceIdentityDerivationDenied,
    UnsupportedHistoricalPathRequest,
    IncompatibleBasisPathPair,
    ReplayNotPermitted,
    RetentionUnavailable,
    ReconstructionNotAdmitted,
    HiddenPathSubstitutionDenied,
    UnsupportedBridgeMaterializationPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoricalEvaluationError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    UnsupportedHistoricalPathRequest {
        requested_path_class: RequestedHistoricalPathClass,
        reason: &'static str,
    },
    IncompatibleBasisPathPair {
        requested_basis_identity: String,
        descriptor_basis_identity: String,
        requested_path_class: RequestedHistoricalPathClass,
    },
    ReplayNotPermitted {
        requested_path_class: RequestedHistoricalPathClass,
    },
    RetentionUnavailable {
        requested_path_class: RequestedHistoricalPathClass,
    },
    ReconstructionNotAdmitted {
        requested_path_class: RequestedHistoricalPathClass,
    },
    HiddenPathSubstitutionDenied {
        requested_path_class: RequestedHistoricalPathClass,
        admitted_path_class: AdmittedHistoricalPathClass,
        attempted_resolved_path_class: ResolvedHistoricalPathClass,
    },
    UnsupportedBridgeMaterializationPath {
        requested_path_class: RequestedHistoricalPathClass,
        path_name: &'static str,
    },
}

impl HistoricalEvaluationError {
    pub fn failure_class(&self) -> HistoricalEvaluationFailureClass {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => {
                HistoricalEvaluationFailureClass::EvidenceIdentityDerivationDenied
            }

            Self::UnsupportedHistoricalPathRequest { .. } => {
                HistoricalEvaluationFailureClass::UnsupportedHistoricalPathRequest
            }
            Self::IncompatibleBasisPathPair { .. } => {
                HistoricalEvaluationFailureClass::IncompatibleBasisPathPair
            }
            Self::ReplayNotPermitted { .. } => HistoricalEvaluationFailureClass::ReplayNotPermitted,
            Self::RetentionUnavailable { .. } => {
                HistoricalEvaluationFailureClass::RetentionUnavailable
            }
            Self::ReconstructionNotAdmitted { .. } => {
                HistoricalEvaluationFailureClass::ReconstructionNotAdmitted
            }
            Self::HiddenPathSubstitutionDenied { .. } => {
                HistoricalEvaluationFailureClass::HiddenPathSubstitutionDenied
            }
            Self::UnsupportedBridgeMaterializationPath { .. } => {
                HistoricalEvaluationFailureClass::UnsupportedBridgeMaterializationPath
            }
        }
    }

    pub fn requested_path_class(&self) -> Option<&RequestedHistoricalPathClass> {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => None,

            Self::UnsupportedHistoricalPathRequest {
                requested_path_class,
                ..
            }
            | Self::IncompatibleBasisPathPair {
                requested_path_class,
                ..
            }
            | Self::ReplayNotPermitted {
                requested_path_class,
                ..
            }
            | Self::RetentionUnavailable {
                requested_path_class,
                ..
            }
            | Self::ReconstructionNotAdmitted {
                requested_path_class,
                ..
            }
            | Self::HiddenPathSubstitutionDenied {
                requested_path_class,
                ..
            }
            | Self::UnsupportedBridgeMaterializationPath {
                requested_path_class,
                ..
            } => Some(requested_path_class),
        }
    }

    pub fn admitted_path_class(&self) -> Option<&AdmittedHistoricalPathClass> {
        match self {
            Self::HiddenPathSubstitutionDenied {
                admitted_path_class,
                ..
            } => Some(admitted_path_class),
            _ => None,
        }
    }

    pub fn attempted_resolved_path_class(&self) -> Option<&ResolvedHistoricalPathClass> {
        match self {
            Self::HiddenPathSubstitutionDenied {
                attempted_resolved_path_class,
                ..
            } => Some(attempted_resolved_path_class),
            _ => None,
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
Self::EvidenceIdentityDerivationDenied(_) => "historical evidence identity exceeded its digest bounds",

            Self::UnsupportedHistoricalPathRequest { reason, .. } => reason,
            Self::IncompatibleBasisPathPair { .. } => {
                "historical evaluation basis identity does not match the lowered capability basis"
            }
            Self::ReplayNotPermitted { .. } => {
                "historical replay was requested but the admitted bridge policy does not permit replay"
            }
            Self::RetentionUnavailable { .. } => {
                "retained-snapshot evaluation was requested but the admitted bridge policy requires historical lookup"
            }
            Self::ReconstructionNotAdmitted { .. } => {
                "full reconstruction was requested but the lower runtime did not admit a reconstruction-compatible lane"
            }
            Self::HiddenPathSubstitutionDenied { .. } => {
                "historical materialization would have changed the admitted path class without explicit authorization"
            }
            Self::UnsupportedBridgeMaterializationPath { .. } => {
                "bridge historical materialization path cannot be lowered into the phase 3 resolved path family"
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn denial_cost_posture(&self) -> HistoricalPathCostPosture {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => {
                HistoricalPathCostPosture::EvidenceIdentityDerivationDenied
            }

            Self::ReplayNotPermitted { .. }
            | Self::RetentionUnavailable { .. }
            | Self::ReconstructionNotAdmitted { .. }
            | Self::IncompatibleBasisPathPair { .. } => {
                HistoricalPathCostPosture::HistoricalPathDeniedByCompatibility
            }
            Self::HiddenPathSubstitutionDenied { .. } => {
                HistoricalPathCostPosture::HistoricalPathSubstitutionDenied
            }
            Self::UnsupportedHistoricalPathRequest { .. }
            | Self::UnsupportedBridgeMaterializationPath { .. } => {
                HistoricalPathCostPosture::HistoricalPathDeniedByUnsupportedPath
            }
        }
    }
}

impl From<IdentityDerivationDenial> for HistoricalEvaluationError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
