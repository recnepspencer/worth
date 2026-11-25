use super::super::WorthQueryPrimaryGraphInstallationDenial;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionDenial, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledPackageIndexDenial, WorthQueryPortablePackageValidationDenial,
};

/// The exact phase that prevented an application from being published.
#[derive(Debug)]
pub enum WorthQueryApplicationPreparationDenial {
    Package(WorthQueryPortablePackageValidationDenial),
    Admission(WorthQueryInstallationAdmissionDenial),
    Runtime(WorthQueryInstalledPackageIndexDenial),
    Schema(WorthQueryInstalledApplicationSchemaDenial),
    Contributions(WorthQueryPrimaryGraphInstallationDenial),
    Graph(WorthQueryPrimaryGraphInstallationDenial),
    InitialState(WorthQueryPrimaryGraphInstallationDenial),
    Publication(WorthQueryPrimaryGraphInstallationDenial),
    ConditionalPublication(
        super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
    ),
    Program(worth_query_installation::facade::WorthQueryApplicationProgramInstallationDenial),
    ConditionalProgramMismatch,
    WorkflowAuthorityRequiresProgram,
    /// Program support was admitted for this installation, but the installed
    /// initial program never reached the runtime that was to carry it.
    ProgramAdmissionIncomplete,
}

impl std::fmt::Display for WorthQueryApplicationPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application preparation denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationPreparationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Package(error) => Some(error),
            Self::Admission(_)
            | Self::Runtime(_)
            | Self::ConditionalProgramMismatch
            | Self::ProgramAdmissionIncomplete => None,
            Self::WorkflowAuthorityRequiresProgram => None,
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
