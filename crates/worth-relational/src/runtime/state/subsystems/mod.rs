pub(crate) use history::{
    OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial, OriginalEmptySchemaInstallation,
    OriginalEmptySchemaObservation, RetainedBranchTransition,
};
pub(crate) use record_identity::MutationRecordAllocations;
mod commit_strategies;
mod durability;
mod history;
mod indexing;
mod lineage;
mod lineage_identity;
mod publication;
mod publication_diagnostics;
mod record_identity;
mod schema_contract_runtime;
pub(crate) mod services;
pub(crate) mod storage;
mod storage_lifecycle;
mod visibility;

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
pub use history::canonical_publication_routes::{
    RelationalCanonicalPositionAdmissionDenial, RelationalPatchPositionReservationCounters,
};
pub use history::history_costs::RelationalBranchSharingCostCounters;
pub use history::RelationalPhase4ReferenceCostCounters;
pub(crate) use history::{authored_write_commit_denial, AuthoredWriteGeneration};
pub(crate) use history::{
    readmit_positioned_canonical_commit, BranchHeadVersionIndexAuthority,
    BranchTransitionPublication, BranchTransitionRecordError, CanonicalCheckpointAdmissionError,
    CanonicalPositionAdmission, CanonicalPublicationRecordError, HistorySubsystem,
    PatchPositionReservationAdmission, PerformedCheckpointSelection,
    PreparedCanonicalPublicationRoute, PreparedCommitSequence,
    PreparedRecoveredVersionedArtifactPublication, PreparedVersionedArtifactAccelerators,
    PreparedVersionedArtifactPublication, RelationalCanonicalPublicationRoutes,
    RelationalForkMaterializationCost, RelationalForkOwnerBinding, RelationalPerformedSourceWalk,
    RelationalPreparationHistory, RelationalSourceHistoryBinding, RetainedCanonicalPosition,
};
pub(crate) use indexing::{
    CapturedIndexDefinitionCatalog, FrozenCheckpointIndexes, IndexCatalogCoverage,
    IndexDefinitionCatalog, IndexDefinitionReadBinding, IndexSelectionState, IndexingState,
    IndexingSubsystem, OriginalCheckpointIndexes, PreparedGenerationBatch,
    PreparedIndexBasisSelection,
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
pub(crate) use publication_diagnostics::{
    RelationalDiagnosticArtifactStore, RelationalDiagnosticOperationCapture,
};
pub(crate) use record_identity::{
    PendingRecordAllocations, ReclaimedRecordSlot, RecordIdentitySubsystem,
};
pub(crate) use schema_contract_runtime::SchemaContractRuntimeSubsystem;
pub(crate) use services::{
    ColdSymbolDirectoryDenial, MutationSymbolMeanings, PendingSymbolMeanings,
    PreparedSymbolPublication, RuntimeInstrumentation, RuntimeServices, RuntimeSymbolTable,
    SymbolReservationDenial,
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
