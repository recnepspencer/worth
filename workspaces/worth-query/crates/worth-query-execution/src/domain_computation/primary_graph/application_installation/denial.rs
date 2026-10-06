use super::super::{WorthQueryHomeAbsent, WorthQueryPrimaryGraphInstallationDenial};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionDenial, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledPackageIndexDenial, WorthQueryPortablePackageValidationDenial,
};

/// The two entries an application opens through, and the two kinds of image
/// they leave. An image with a program activation is a program image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOpenEntryKind {
    Declaration,
    Program,
}

/// The exact phase that prevented an application from being opened.
#[derive(Debug)]
#[non_exhaustive]
pub enum WorthQueryApplicationOpenDenial {
    /// The home's form cannot be opened yet. Nothing was read from it.
    Home(WorthQueryHomeAbsent),
    /// The home holds an image the other entry wrote.
    EntryKindMismatch {
        image: WorthQueryOpenEntryKind,
        entry: WorthQueryOpenEntryKind,
    },
    Package(WorthQueryPortablePackageValidationDenial),
    Admission(WorthQueryInstallationAdmissionDenial),
    Runtime(WorthQueryInstalledPackageIndexDenial),
    Schema(WorthQueryInstalledApplicationSchemaDenial),
    Contributions(WorthQueryPrimaryGraphInstallationDenial),
    Graph(WorthQueryPrimaryGraphInstallationDenial),
    /// Native publication performed, but settlement stopped without repair custody.
    /// No World or acknowledged successor was issued.
    AdoptionSettlementFailed(Box<worth_relational::facade::transactions::TransactionCommitError>),
    /// Native publication performed, but its settlement deferred. The refused
    /// home holds the repair capsule; this is the cause.
    AdoptionDeferred(String),
    /// Native target settlement acknowledged, but image capture stopped. The
    /// refused home holds the repair capsule; this is the cause.
    AdoptionCaptureStopped(String),
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

impl std::fmt::Display for WorthQueryApplicationOpenDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application open denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationOpenDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Package(error) => Some(error),
            Self::Home(_)
            | Self::EntryKindMismatch { .. }
            | Self::Admission(_)
            | Self::Runtime(_)
            | Self::ConditionalProgramMismatch
            | Self::ProgramAdmissionIncomplete
            | Self::RequiredOutputSourceAction(_) => None,
            Self::WorkflowAuthorityRequiresProgram => None,
            Self::AdoptionDeferred(_)
            | Self::AdoptionCaptureStopped(_)
            | Self::AdoptionSettlementFailed(_) => None,
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
