//! Opening refusals keep the exact Query preparation and physical Home phase.
use super::runtime::DirectoryHome;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::*,
    application_installation::WorthQueryApplicationPreparationDenial,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_store_application::*;

mod query_custody;
mod unclean;
pub use query_custody::{UnadmittedHomeQueryCustodyRefused, UnadmittedHomeQueryService};

pub enum WorthQueryApplicationHomeOpenRefused<Schema: ApplicationSchemaComposition, Program> {
    TransitionProgrammeHandoff {
        query: Box<PreparedExistingHomeSignalProgram<Schema, Program>>,
        home: Box<super::runtime::DirectoryHome>,
        handoff: Box<worth_store_application::ApplicationHomeProgrammeHandoffRefused>,
        _bindings: Box<QueryHomeAuthoredBindings>,
    },

    Preparation(WorthQueryApplicationPreparationDenial),
    TransitionSignalProgramPreparation {
        home: Box<DirectoryHome>,
        query: Box<QueryHomeSignalProgramPreparationRefused<Schema, Program>>,
    },
    TransitionComposition {
        query: Box<ReadmittedExistingHomeNativeMeaning<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeQueryCompositionDenial>,
    },
    SignalProgramPreparation {
        home: Box<PreparedQueryHomeSource>,
        query: Box<QueryHomeSignalProgramPreparationRefused<Schema, Program>>,
    },
    Binding {
        query: Box<PreparedHomeApplication<Schema, Program>>,
        cause: Box<worth_query_installation::facade::WorthQueryHomeBindingDenial>,
    },
    Physical {
        query: Box<PreparedHomeApplication<Schema, Program>>,
        cause: ApplicationHomeOpenRefusal,
    },
    RelationalAttachment {
        query: Box<PreparedHomeApplication<Schema, Program>>,
        home: Box<RelationalAttachmentRefused>,
    },
    NativePreparation {
        query: Box<PreparedHomeApplication<Schema, Program>>,
        home: Box<UnrecoveredApplicationHome>,
        cause: Box<ApplicationHomeNativeSchemaPreparationDenial>,
    },
    NativeInstallation {
        query: Box<QueryHomeNativeInstallationRefused<Schema, Program>>,
        home: Box<UnrecoveredApplicationHome>,
    },
    NativeReadmission {
        query: Box<PreparedHomeNativeMeaning<Schema, Program>>,
        home: Box<RelationalHomeReadmissionRefused>,
    },
    QueryPublicationReadmissionRequired {
        query: Box<PreparedHomeNativeMeaning<Schema, Program>>,
        home: Box<ApplicationHomeInitializationRefused>,
    },
    NativeFinalizationInput {
        query: Box<PreparedHomeNativeMeaning<Schema, Program>>,
        home: Box<UnsharedReadmittedApplicationHome>,
        cause: Box<ApplicationHomeOpenRefusal>,
    },
    ExistingNativeFinalization {
        query: Box<QueryExistingHomeNativeReadmissionRefused<Schema, Program>>,
        home: Box<UnsharedReadmittedApplicationHome>,
    },
    ExistingBridgeDefinition {
        query: Box<ReadmittedExistingHomeNativeMeaning<Schema, Program>>,
        home: Box<UnsharedReadmittedApplicationHome>,
        cause: Box<WorthQueryPrimaryGraphInstallationDenial>,
    },
    ExistingWorldSourcePreparation {
        query: Box<ReadmittedExistingHomeNativeMeaning<Schema, Program>>,
        home: Box<WorldHomeReadmissionRefused>,
    },
    ExistingWorldReadmission {
        query: Box<PreparedExistingHomeSignalProgram<Schema, Program>>,
        _bindings: Box<QueryHomeAuthoredBindings>,
        home: Box<WorldHomeReadmissionRefused>,
    },
    ExistingComposition {
        query: Box<PreparedExistingHomeSignalProgram<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeQueryCompositionDenial>,
    },
    ExistingSourceAttachment {
        query: Box<QueryExistingHomeSourceAttachmentRefused<Schema, Program>>,
        home: Box<DirectoryHome>,
    },
    ExistingCheckpointMaintenance {
        query: Box<PreparedExistingHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeCheckpointDenial>,
    },
    ExistingWorldRoot {
        query: Box<PreparedExistingHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: ApplicationHomeQueryRootDenial,
    },
    ExistingQueryAttachment {
        query: Box<QueryExistingHomeWorldAttachmentRefused<Schema, Program>>,
        home: Box<DirectoryHome>,
    },
    NativeFinalization {
        query: Box<QueryHomeNativeReadmissionRefused<Schema, Program>>,
        home: Box<UnsharedReadmittedApplicationHome>,
    },
    BridgeDefinition {
        query: Box<ReadmittedHomeNativeMeaning<Schema, Program>>,
        home: Box<UnsharedReadmittedApplicationHome>,
        cause: Box<WorthQueryPrimaryGraphInstallationDenial>,
    },
    WorldSourcePreparation {
        query: Box<ReadmittedHomeNativeMeaning<Schema, Program>>,
        home: Box<WorldHomeReadmissionRefused>,
    },
    WorldReadmission {
        query: Box<PreparedHomeSignalProgram<Schema, Program>>,
        _bindings: Box<QueryHomeAuthoredBindings>,
        home: Box<WorldHomeReadmissionRefused>,
    },
    Composition {
        query: Box<PreparedHomeSignalProgram<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeQueryCompositionDenial>,
    },
    SourceAttachment {
        query: Box<QueryHomeSourceAttachmentRefused<Schema, Program>>,
        home: Box<DirectoryHome>,
    },
    InitialState {
        query: Box<PreparedHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: QueryHomeInitialStateDenial,
    },
    SeedPublication {
        query: Box<PreparedHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<WorthQueryPrimaryGraphInstallationDenial>,
    },
    CheckpointMaintenance {
        query: Box<PreparedHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeCheckpointDenial>,
    },
    WorldRoot {
        query: Box<PreparedHomeQueryRuntime<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: Box<ApplicationHomeQueryRootDenial>,
    },
    QueryAttachment {
        query: Box<QueryHomeWorldAttachmentRefused<Schema, Program>>,
        home: Box<DirectoryHome>,
    },
    CustodyReadmission {
        query: Box<UnadmittedHomeQueryService<Schema, Program>>,
        home: Box<DirectoryHome>,
        cause: HomeCustodyReadmissionDenial,
    },
    OwnerAdmission {
        query: Box<UnadmittedHomeQueryService<Schema, Program>>,
        home: Box<ApplicationHomeOwnerAdmissionRefused>,
    },
}

#[derive(Debug)]
pub enum HomeCustodyReadmissionDenial {
    Query(QueryHomeCustodyReadmissionDenial),
    /// The Home refused Query's decoded dispatch prefix as its actual journal head.
    DispatchJournal(ApplicationHomeDispatchJournalDenial),
}

impl<Schema: ApplicationSchemaComposition, Program> std::fmt::Debug
    for WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WorthQueryApplicationHomeOpenRefused")
            .field(&self.detail())
            .finish()
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    pub fn detail(&self) -> String {
        match self {
            Self::TransitionProgrammeHandoff { handoff, .. } => {
                format!("authored programme handoff refused: {:?}", handoff.reason())
            }

            Self::SignalProgramPreparation { query, .. } => format!("{:?}", query.reason()),
            Self::TransitionSignalProgramPreparation { query, .. } => {
                format!("{:?}", query.reason())
            }
            Self::TransitionComposition { cause, .. } => format!("{cause:?}"),
            Self::Preparation(cause) => format!("{cause:?}"),
            Self::Binding { cause, .. } => format!("{cause:?}"),
            Self::Physical { cause, .. } => format!("{cause:?}"),
            Self::RelationalAttachment { home, .. } => format!("{home:?}"),
            Self::NativePreparation { cause, .. } => format!("{cause:?}"),
            Self::NativeInstallation { query, .. } => query.reason().to_string(),
            Self::NativeReadmission { home, .. } => format!("{:?}", home.reason()),
            Self::QueryPublicationReadmissionRequired { home, .. } => {
                format!("{:?}", home.reason())
            }
            Self::NativeFinalizationInput { cause, .. } => format!("{cause:?}"),
            Self::ExistingNativeFinalization { query, .. } => query.reason().to_string(),
            Self::ExistingBridgeDefinition { cause, .. } => cause.to_string(),
            Self::ExistingWorldSourcePreparation { home, .. } => format!("{:?}", home.reason()),
            Self::ExistingWorldReadmission { home, .. } => format!("{:?}", home.reason()),
            Self::ExistingComposition { cause, .. } => format!("{cause:?}"),
            Self::ExistingSourceAttachment { query, .. } => query.reason().to_string(),
            Self::ExistingCheckpointMaintenance { cause, .. } => format!("{cause:?}"),
            Self::ExistingWorldRoot { cause, .. } => format!("{cause:?}"),
            Self::ExistingQueryAttachment { query, .. } => query.detail(),
            Self::NativeFinalization { query, .. } => query.reason().to_string(),
            Self::BridgeDefinition { cause, .. } => cause.to_string(),
            Self::WorldSourcePreparation { home, .. } => format!("{:?}", home.reason()),
            Self::WorldReadmission { home, .. } => format!("{:?}", home.reason()),
            Self::Composition { cause, .. } => format!("{cause:?}"),
            Self::SourceAttachment { query, .. } => query.reason().to_string(),
            Self::InitialState { cause, .. } => format!("{cause:?}"),
            Self::SeedPublication { cause, .. } => cause.to_string(),
            Self::CheckpointMaintenance { cause, .. } => format!("{cause:?}"),
            Self::WorldRoot { cause, .. } => format!("{cause:?}"),
            Self::QueryAttachment { query, .. } => query.detail(),
            Self::CustodyReadmission { cause, .. } => format!("{cause:?}"),
            Self::OwnerAdmission { home, .. } => format!("{:?}", home.reason()),
        }
    }
}
