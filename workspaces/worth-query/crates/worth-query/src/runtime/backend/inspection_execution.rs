use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::CausalInspectionMaterializationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBackendInspectionErrorKind {
    EvidenceIdentityDerivationDenied,
    Construction,
    Unavailable,
    Materialization,
}

#[derive(Debug)]
pub enum WorthQueryBackendInspectionError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Construction(
        worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop,
    ),
    Unavailable(WorthQueryWorkspaceError),
    Materialization(CausalInspectionMaterializationError),
}

impl WorthQueryBackendInspectionError {
    pub fn kind(&self) -> WorthQueryBackendInspectionErrorKind {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => {
                WorthQueryBackendInspectionErrorKind::EvidenceIdentityDerivationDenied
            }
            Self::Construction(_) => WorthQueryBackendInspectionErrorKind::Construction,
            Self::Unavailable(_) => WorthQueryBackendInspectionErrorKind::Unavailable,
            Self::Materialization(_) => WorthQueryBackendInspectionErrorKind::Materialization,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::EvidenceIdentityDerivationDenied(cause) => {
                format!("evidence identity refused: {cause:?}")
            }
            Self::Construction(error) => format!("{error:?}"),
            Self::Unavailable(error) => error.to_string(),
            Self::Materialization(error) => format!("{error:?}"),
        }
    }

    pub(crate) fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable(WorthQueryWorkspaceError::new(message))
    }
}

impl From<CausalInspectionMaterializationError> for WorthQueryBackendInspectionError {
    fn from(error: CausalInspectionMaterializationError) -> Self {
        Self::Materialization(error)
    }
}

impl From<IdentityDerivationDenial> for WorthQueryBackendInspectionError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl From<worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop>
    for WorthQueryBackendInspectionError
{
    fn from(
        cause: worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop,
    ) -> Self {
        Self::Construction(cause)
    }
}
