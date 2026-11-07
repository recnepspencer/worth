//! Signal coordinates describe Home data, never process-local authority.
mod branch;
mod definition;
mod journal;
mod occurrence;
mod owner;
pub use branch::{
    DurableSignalBranchIdentity, DurableSignalCellIncarnation, DurableSignalGeneration,
};
pub use definition::{
    SignalBranchAdvanceIdentity, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference,
};
pub use journal::SignalDurableJournal;
pub use occurrence::{
    DurableSignalAdmissionIdentity, DurableSignalConditionalOccurrence,
    DurableSignalSnapshotIdentity,
};
pub use owner::{DurableSignalOwnerIdentity, SignalOwnerSlotIdentity};
