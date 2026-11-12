//! Recover owner source before any live service can be issued.
mod identity_frontiers;
mod source_catalog;
mod source_custody;
mod source_locations;
pub use identity_frontiers::SignalRecordedIdentityContribution;
pub(crate) use source_catalog::SignalOwnerSourceReadmission;
pub(crate) use source_custody::{
    SignalAdvanceLocationReservation, SignalDefinitionLocationReservation,
};
pub(crate) use source_locations::{SignalDefinitionSourceRecord, SignalRecoveredSourceLocations};
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalSnapshotIdentity, SignalBranchAdvanceIdentity,
    SignalDefinitionTransitionIdentity, SignalDefinitionTransitionReference,
};

#[derive(Debug)]
pub enum SignalOwnerReadmissionDenial {
    Source(crate::branch::owner_services::SignalDurableSourceDenial),
    Medium(super::SignalDurableMediumDenial),
    Frame(super::SignalDurableFrameDenial),
    Image(super::SignalAuthoredImageDenial),
    ForeignOwner,
    InstallationRefused(DurableSignalBranchIdentity),
    ProgramAlreadyServing,
    PublicationStatePoisoned,
    OrderedStorage(worth_execution::ExecutionOrderedMapDenial),
    Request(worth_execution::WorkCeilingDenial),
    Memory(worth_execution::MemoryLimitDenial),
    MetadataCapacity,
    PublicationNotQuiescent(super::SignalPublicationTaskPosture),
    NoBranches,
    MissingDefinition(SignalDefinitionTransitionReference),
    DuplicateDefinition(SignalDefinitionTransitionIdentity),
    DuplicateAdvance(SignalBranchAdvanceIdentity),
    MissingAdvance(SignalBranchAdvanceIdentity),
    SourcePointMismatch(worth_foundational::facade::SignalBranchAdvanceReference),
    MissingBranch(DurableSignalBranchIdentity),
    DuplicateBranch(DurableSignalBranchIdentity),
    DuplicateBranchName(String),
    BranchChainMismatch(DurableSignalBranchIdentity),
    SnapshotRequired,
    MissingSnapshot(DurableSignalSnapshotIdentity),
    DuplicateSnapshot(DurableSignalSnapshotIdentity),
    Capacity { branches: usize, snapshots: usize },
    IdentityExhausted,
    ConditionalOccurrenceMismatch(worth_foundational::facade::DurableSignalNodeIdentity),
    OwnerServices(crate::branch::SignalOwnerServiceIssuanceDenial),
}
