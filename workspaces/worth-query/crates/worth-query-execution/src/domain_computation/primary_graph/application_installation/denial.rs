use super::super::WorthQueryPrimaryGraphInstallationDenial;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionDenial, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledPackageIndexDenial, WorthQueryPortablePackageValidationDenial,
};

/// The exact phase that prevented an application from being published.
#[derive(Debug)]
pub enum WorthQueryInMemoryApplicationDenial {
    Package(WorthQueryPortablePackageValidationDenial),
    Admission(WorthQueryInstallationAdmissionDenial),
    Runtime(WorthQueryInstalledPackageIndexDenial),
    Schema(WorthQueryInstalledApplicationSchemaDenial),
    Contributions(WorthQueryPrimaryGraphInstallationDenial),
    Graph(WorthQueryPrimaryGraphInstallationDenial),
    /// Native target publication settled; a later installation phase refused.
    /// The checkpoint preserves that actual successor for ordinary target restore.
    CheckpointTransitionAcknowledged {
        checkpoint: super::super::WorthQueryApplicationCheckpoint,
        cause: Box<WorthQueryInMemoryApplicationDenial>,
    },
    /// Native publication performed, but settlement stopped without repair custody.
    /// No World or acknowledged successor was issued.
    CheckpointTransitionSettlementFailed(
        Box<worth_relational::facade::transactions::TransactionCommitError>,
    ),
    CheckpointTransitionDeferred(Box<super::WorthQueryCheckpointTransitionRecovery>),
    /// Native target settlement acknowledged, but checkpoint capture stopped.
    CheckpointTransitionCaptureStopped(Box<super::WorthQueryCheckpointTransitionRecovery>),
    InitialState(WorthQueryPrimaryGraphInstallationDenial),
    Publication(WorthQueryPrimaryGraphInstallationDenial),
    ConditionalPublication(
        super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
    ),
    Program(worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenial),
    ConditionalProgramMismatch,
    RequiredOutputSourceAction(String),
    WorkflowAuthorityRequiresProgram,
    /// Program support was admitted for this installation, but the installed
    /// initial program never reached the runtime that was to carry it.
    ProgramAdmissionIncomplete,
}

impl std::fmt::Display for WorthQueryInMemoryApplicationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "in-memory application installation denied: {self:?}"
        )
    }
}

impl std::error::Error for WorthQueryInMemoryApplicationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Package(error) => Some(error),
            Self::Admission(_)
            | Self::Runtime(_)
            | Self::ConditionalProgramMismatch
            | Self::ProgramAdmissionIncomplete
            | Self::RequiredOutputSourceAction(_) => None,
            Self::WorkflowAuthorityRequiresProgram => None,
            Self::CheckpointTransitionDeferred(_)
            | Self::CheckpointTransitionCaptureStopped(_)
            | Self::CheckpointTransitionSettlementFailed(_) => None,
            Self::CheckpointTransitionAcknowledged { cause, .. } => Some(cause.as_ref()),
            Self::Schema(error) => Some(error),
            Self::Contributions(error)
            | Self::Graph(error)
            | Self::InitialState(error)
            | Self::Publication(error) => Some(error),
            Self::ConditionalPublication(_) => None,
            Self::Program(error) => Some(error),
        }
    }
}
