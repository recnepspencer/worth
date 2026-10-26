mod commit_strategies;
mod durability;
mod history;
mod indexing;
mod lineage;
mod lineage_identity;
mod owned_state;
mod publication;
mod publication_diagnostics;
mod record_identity;
mod schema_contract_runtime;
mod services;
mod storage;
mod visibility;

pub(crate) trait RuntimeSubsystem: Sized {
    type Config;

    fn new(config: &Self::Config) -> Self;
    fn fork(&self) -> Self;
}

pub(crate) use commit_strategies::CommitStrategiesSubsystem;
pub(crate) use durability::{DurabilitySubsystem, RelationalCheckpointManifestPublication};
pub(crate) use history::{
    readmit_positioned_canonical_commit, BranchHeadVersionIndexAuthority,
    BranchTransitionPublication, BranchTransitionRecordError, CanonicalCheckpointAdmissionError,
    CanonicalPositionAdmission, CanonicalPublicationRecordError, HistorySubsystem,
    PatchPositionReservationAdmission, PerformedCheckpointSelection,
    PreparedCanonicalPublicationRoute, PreparedRecoveredVersionedArtifactPublication,
    PreparedVersionedArtifactAccelerators, PreparedVersionedArtifactPublication,
    RelationalCanonicalPublicationRoutes, RelationalForkMaterializationCost,
    RelationalForkOwnerBinding, RelationalPreparationHistory,
};
pub use history::{RelationalBranchSharingCostCounters, RelationalPhase4ReferenceCostCounters};
pub use history::{
    RelationalCanonicalPositionAdmissionDenial, RelationalPatchPositionReservationCounters,
};
pub(crate) use indexing::{
    IndexDefinitionReadBinding, IndexingState, IndexingSubsystem, PreparedGenerationBatch,
};
pub(crate) use lineage::{LineageState, LineageSubsystem, ValidatedLineageEventBatch};
pub(crate) use lineage_identity::LineageIdentityAllocator;
pub(crate) use owned_state::RuntimeOwnedState;
pub(crate) use publication::PublicationSubsystem;
pub(crate) use publication_diagnostics::RelationalDiagnosticArtifactStore;
pub(crate) use record_identity::{
    PendingRecordAllocations, ReclaimedRecordSlot, RecordIdentitySubsystem,
};
pub(crate) use schema_contract_runtime::SchemaContractRuntimeSubsystem;
pub(crate) use services::{RuntimeInstrumentation, RuntimeServices, RuntimeSymbolTable};
pub(crate) use storage::{
    MirrorResidencyDemandBinding, PartitionEdition, PartitionEditionCopyLane,
    PartitionEditionWriter, StorageSubsystem,
};
pub(crate) use visibility::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotCloseout, PublishedSnapshotReservationDenial,
    PublishedSnapshotSlotReservation, ReplayRetentionState, SnapshotHandleBinding,
    VisibilityResidency, VisibilitySubsystem,
};
