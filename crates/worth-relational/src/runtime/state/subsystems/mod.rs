mod commit_strategies;
pub(crate) mod durability;
pub(crate) mod history;
pub(crate) mod indexing;
pub(crate) mod lineage;
pub(crate) mod lineage_identity;
pub(crate) mod publication;
pub(crate) use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity;
mod schema_contract_construction;
pub(crate) mod schema_contract_runtime;
pub(crate) mod services;
pub(crate) use worth_relational_storage::internal_port::runtime::state::subsystems::storage;
mod storage_lifecycle;
pub(crate) mod visibility;

pub(crate) trait RuntimeSubsystem: Sized {
    type Config;

    fn new(config: &Self::Config) -> Self;
    fn fork(&self) -> Self;
}

pub(crate) use crate::storage::substrate::owned_state::RuntimeOwnedState;
pub(crate) use commit_strategies::CommitStrategiesSubsystem;
pub(crate) use durability::{DurabilitySubsystem, RelationalCheckpointManifestSelection};
pub(crate) use history::canonical_publication_routes::{
    BranchTransitionPublication, CanonicalCheckpointAdmissionError,
    OriginalBranchSchemaInputsDenial, PerformedCheckpointSelection,
    RelationalCanonicalPublicationRoutes, RelationalPerformedSourceWalk,
};
pub use history::canonical_publication_routes::{
    RelationalCanonicalPositionAdmissionDenial, RelationalPatchPositionReservationCounters,
};
pub use history::history_costs::RelationalBranchSharingCostCounters;
pub(crate) use history::history_preparation_binding::RelationalPreparationHistory;
pub(crate) use history::history_source_retirement::RelationalSourceHistoryBinding;
pub use history::RelationalPhase4ReferenceCostCounters;
pub(crate) use history::{
    BranchHeadVersionIndexAuthority, HistorySubsystem, RelationalForkMaterializationCost,
    RelationalForkOwnerBinding,
};
pub(crate) use indexing::IndexDefinitionReadBinding;
pub use indexing::{
    PreparedRelationalIndexDefinitions, RelationalIndexDefinitionPreparation,
    RelationalIndexDefinitionPreparationDenial,
};
pub(crate) use lineage::{LineageNamingBasisBinding, LineageSubsystem, ResidentLineageNode};
#[cfg(test)]
pub(crate) use lineage::{LineageState, ValidatedLineageEventBatch};
pub(crate) use lineage_identity::LineageIdentityAllocator;
pub(crate) use publication::PublicationSubsystem;
pub(crate) use record_identity::{ReclaimedRecordSlot, RecordIdentitySubsystem};
pub(crate) use schema_contract_runtime::SchemaContractRuntimeSubsystem;
#[cfg(test)]
pub(crate) use services::SymbolReservationDenial;
pub(crate) use services::{
    PendingSymbolMeanings, RuntimeInstrumentation, RuntimeServices, RuntimeSymbolTable,
};
pub(crate) use storage::{
    MirrorResidencyDemandBinding, PartitionEdition, PartitionEditionCopyLane,
    PartitionEditionWriter, ResidentPartitionEdition, ResidentPartitionEntry, StorageSubsystem,
};
pub(crate) use visibility::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotReservationDenial,
    PublishedSnapshotSlotReservation, SnapshotHandleBinding, VisibilitySubsystem,
};

pub use record_identity::{
    RecordReservationObservation, RecordReservationObservationDenial, RecordReservationReport,
};

pub use services::RelationalSymbolTableForTest;
pub(crate) use storage::TryPartitionMutationDenied;
