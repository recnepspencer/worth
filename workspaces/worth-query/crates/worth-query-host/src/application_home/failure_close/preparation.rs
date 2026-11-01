//! Exact Query preparations retained while the same World refuses close.
use super::*;
pub(super) enum WorldQueryPreparation<Schema: ApplicationSchemaComposition, Program> {
    ExistingComposition {
        _query: ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        _cause: ApplicationHomeQueryCompositionDenial,
    },
    ExistingSource(QueryExistingHomeSourceAttachmentRefused<Schema, Program>),
    ExistingRoot {
        _query: PreparedExistingHomeQueryRuntime<Schema, Program>,
        _cause: ApplicationHomeQueryRootDenial,
    },
    ExistingAttachment(QueryExistingHomeWorldAttachmentRefused<Schema, Program>),
    Composition {
        _query: ReadmittedHomeNativeMeaning<Schema, Program>,
        _cause: ApplicationHomeQueryCompositionDenial,
    },
    Source(QueryHomeSourceAttachmentRefused<Schema, Program>),
    Bootstrap {
        _query: PreparedHomeQueryRuntime<Schema, Program>,
        _cause: BootstrapFailure,
    },
    Attachment(QueryHomeWorldAttachmentRefused<Schema, Program>),
    Custody {
        _query: ReadmittedHomeQueryService<Schema, Program>,
        _cause: HomeCustodyReadmissionDenial,
    },
    OwnerAdmission {
        _query: ReadmittedHomeQueryService<Schema, Program>,
        _cause: ApplicationHomeOwnerAdmissionDenial,
    },
}
pub(super) enum BootstrapFailure {
    InitialState(QueryHomeInitialStateDenial),
    Seed(WorthQueryPrimaryGraphInstallationDenial),
    Root(ApplicationHomeQueryRootDenial),
}
impl BootstrapFailure {
    pub(super) fn detail(&self) -> String {
        match self {
            Self::InitialState(cause) => format!("{cause:?}"),
            Self::Seed(cause) => cause.to_string(),
            Self::Root(cause) => format!("{cause:?}"),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program> WorldQueryPreparation<Schema, Program> {
    pub(super) fn detail(&self) -> String {
        match self {
            Self::ExistingComposition { _cause, .. } => format!("{_cause:?}"),
            Self::ExistingSource(query) => query.reason().to_string(),
            Self::ExistingRoot { _cause, .. } => format!("{_cause:?}"),
            Self::ExistingAttachment(query) => query.detail(),
            Self::Composition { _cause, .. } => format!("{_cause:?}"),
            Self::Source(query) => query.reason().to_string(),
            Self::Bootstrap { _cause, .. } => _cause.detail(),
            Self::Attachment(query) => query.detail(),
            Self::Custody { _cause, .. } => format!("{_cause:?}"),
            Self::OwnerAdmission { _cause, .. } => format!("{_cause:?}"),
        }
    }
}
