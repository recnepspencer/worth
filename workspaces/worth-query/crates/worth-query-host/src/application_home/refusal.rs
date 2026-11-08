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
    Preparation(WorthQueryApplicationPreparationDenial),
    Binding {
        query: PreparedHomeApplication<Schema, Program>,
        cause: worth_query_installation::facade::WorthQueryHomeBindingDenial,
    },
    Physical {
        query: PreparedHomeApplication<Schema, Program>,
        cause: ApplicationHomeOpenRefusal,
    },
    RelationalAttachment {
        query: PreparedHomeApplication<Schema, Program>,
        home: RelationalAttachmentRefused,
    },
    NativePreparation {
        query: PreparedHomeApplication<Schema, Program>,
        home: UnrecoveredApplicationHome,
        cause: ApplicationHomeNativeSchemaPreparationDenial,
    },
    NativeInstallation {
        query: QueryHomeNativeInstallationRefused<Schema, Program>,
        home: UnrecoveredApplicationHome,
    },
    NativeReadmission {
        query: PreparedHomeNativeMeaning<Schema, Program>,
        home: RelationalHomeReadmissionRefused,
    },
    QueryPublicationReadmissionRequired {
        query: PreparedHomeNativeMeaning<Schema, Program>,
        home: ApplicationHomeInitializationRefused,
    },
    NativeFinalizationInput {
        query: PreparedHomeNativeMeaning<Schema, Program>,
        home: UnsharedReadmittedApplicationHome,
        cause: ApplicationHomeOpenRefusal,
    },
    ExistingNativeFinalization {
        query: QueryExistingHomeNativeReadmissionRefused<Schema, Program>,
        home: UnsharedReadmittedApplicationHome,
    },
    ExistingBridgeDefinition {
        query: ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        home: UnsharedReadmittedApplicationHome,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    },
    ExistingWorldReadmission {
        query: ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        home: WorldHomeReadmissionRefused,
    },
    ExistingComposition {
        query: ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeQueryCompositionDenial,
    },
    ExistingSourceAttachment {
        query: QueryExistingHomeSourceAttachmentRefused<Schema, Program>,
        home: DirectoryHome,
    },
    ExistingCheckpointMaintenance {
        query: PreparedExistingHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeCheckpointDenial,
    },
    ExistingWorldRoot {
        query: PreparedExistingHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeQueryRootDenial,
    },
    ExistingQueryAttachment {
        query: QueryExistingHomeWorldAttachmentRefused<Schema, Program>,
        home: DirectoryHome,
    },
    NativeFinalization {
        query: QueryHomeNativeReadmissionRefused<Schema, Program>,
        home: UnsharedReadmittedApplicationHome,
    },
    BridgeDefinition {
        query: ReadmittedHomeNativeMeaning<Schema, Program>,
        home: UnsharedReadmittedApplicationHome,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    },
    WorldReadmission {
        query: ReadmittedHomeNativeMeaning<Schema, Program>,
        home: WorldHomeReadmissionRefused,
    },
    Composition {
        query: ReadmittedHomeNativeMeaning<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeQueryCompositionDenial,
    },
    SourceAttachment {
        query: QueryHomeSourceAttachmentRefused<Schema, Program>,
        home: DirectoryHome,
    },
    InitialState {
        query: PreparedHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: QueryHomeInitialStateDenial,
    },
    SeedPublication {
        query: PreparedHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: WorthQueryPrimaryGraphInstallationDenial,
    },
    CheckpointMaintenance {
        query: PreparedHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeCheckpointDenial,
    },
    WorldRoot {
        query: PreparedHomeQueryRuntime<Schema, Program>,
        home: DirectoryHome,
        cause: ApplicationHomeQueryRootDenial,
    },
    QueryAttachment {
        query: QueryHomeWorldAttachmentRefused<Schema, Program>,
        home: DirectoryHome,
    },
    CustodyReadmission {
        query: UnadmittedHomeQueryService<Schema, Program>,
        home: DirectoryHome,
        cause: HomeCustodyReadmissionDenial,
    },
    OwnerAdmission {
        query: UnadmittedHomeQueryService<Schema, Program>,
        home: ApplicationHomeOwnerAdmissionRefused,
    },
}

#[derive(Debug)]
pub enum HomeCustodyReadmissionDenial {
    Query(QueryHomeCustodyReadmissionDenial),
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
            Self::ExistingWorldReadmission { home, .. } => format!("{:?}", home.reason()),
            Self::ExistingComposition { cause, .. } => format!("{cause:?}"),
            Self::ExistingSourceAttachment { query, .. } => query.reason().to_string(),
            Self::ExistingCheckpointMaintenance { cause, .. } => format!("{cause:?}"),
            Self::ExistingWorldRoot { cause, .. } => format!("{cause:?}"),
            Self::ExistingQueryAttachment { query, .. } => query.detail(),
            Self::NativeFinalization { query, .. } => query.reason().to_string(),
            Self::BridgeDefinition { cause, .. } => cause.to_string(),
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
