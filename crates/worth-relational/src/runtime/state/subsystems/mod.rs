pub(crate) use record_identity::MutationRecordAllocations;
mod commit_strategies;
pub(crate) mod durability;
pub(crate) mod history;
pub(crate) mod indexing;
pub(crate) mod lineage;
pub(crate) mod lineage_identity;
pub(crate) mod publication;
pub(crate) mod record_identity;
mod schema_contract_runtime;
pub(crate) mod services;
pub(crate) mod storage;
mod storage_lifecycle;
pub(crate) mod visibility;

pub(crate) trait RuntimeSubsystem: Sized {
    type Config;

    fn new(config: &Self::Config) -> Self;
    fn fork(&self) -> Self;
}

pub(crate) use crate::storage::substrate::owned_state::RuntimeOwnedState;
pub(crate) use commit_strategies::CommitStrategiesSubsystem;
pub(crate) use durability::{
    DurabilitySubsystem, RelationalCheckpointManifestPublication,
    RelationalCheckpointManifestSelection,
};
pub(crate) use history::canonical_publication_routes::{
    authored_write_commit_denial, readmit_positioned_canonical_commit, BranchTransitionPublication,
    BranchTransitionRecordError, CanonicalCheckpointAdmissionError,
    CanonicalPublicationRecordError, OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial,
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation,
    PatchPositionReservationAdmission, PerformedCheckpointSelection,
    PreparedCanonicalPublicationRoute, RelationalCanonicalPublicationRoutes,
    RelationalPerformedSourceWalk, RetainedBranchTransition,
};
pub use history::canonical_publication_routes::{
    RelationalCanonicalPositionAdmissionDenial, RelationalPatchPositionReservationCounters,
};
pub use history::history_costs::RelationalBranchSharingCostCounters;
pub(crate) use history::history_preparation_binding::RelationalPreparationHistory;
pub(crate) use history::history_source_retirement::RelationalSourceHistoryBinding;
pub use history::RelationalPhase4ReferenceCostCounters;
pub(crate) use history::{
    BranchHeadVersionIndexAuthority, HistorySubsystem, PreparedCommitSequence,
    PreparedRecoveredVersionedArtifactPublication, PreparedVersionedArtifactAccelerators,
    PreparedVersionedArtifactPublication, RelationalForkMaterializationCost,
    RelationalForkOwnerBinding,
};
pub(crate) use indexing::{
    CapturedIndexDefinitionCatalog, IndexCatalogCoverage, IndexDefinitionReadBinding,
    IndexingState, OriginalCheckpointIndexes,
};
pub use indexing::{
    PreparedRelationalIndexDefinitions, RelationalIndexDefinitionPreparation,
    RelationalIndexDefinitionPreparationDenial,
};
pub(crate) use lineage::{
    LineageNamingBasisBinding, LineageState, LineageSubsystem, ResidentLineageEventPage,
    ResidentLineageNode, ValidatedLineageEventBatch,
};
pub(crate) use lineage_identity::LineageIdentityAllocator;
pub(crate) use publication::PublicationSubsystem;
pub(crate) use record_identity::{
    PendingRecordAllocations, ReclaimedRecordSlot, RecordIdentitySubsystem,
};
pub(crate) use schema_contract_runtime::SchemaContractRuntimeSubsystem;
pub(crate) use services::{
    MutationSymbolMeanings, PendingSymbolMeanings, PreparedSymbolPublication,
    RuntimeInstrumentation, RuntimeServices, RuntimeSymbolTable, SymbolReservationDenial,
};
pub(crate) use storage::{
    MirrorResidencyDemandBinding, PartitionEdition, PartitionEditionCopyLane,
    PartitionEditionWriter, ResidentMirrorTarget, ResidentPartitionEdition, ResidentPartitionEntry,
    StorageSubsystem,
};
pub(crate) use visibility::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotCloseout, PublishedSnapshotReleaseObservation,
    PublishedSnapshotReservationDenial, PublishedSnapshotSlotReservation, SnapshotHandleBinding,
    VisibilityResidency, VisibilitySubsystem,
};

pub use record_identity::{
    RecordReservationObservation, RecordReservationObservationDenial, RecordReservationReport,
};

pub use services::RelationalSymbolTableForTest;
pub(crate) use storage::TryPartitionMutationDenied;
