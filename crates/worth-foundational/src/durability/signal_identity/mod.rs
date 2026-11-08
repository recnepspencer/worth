//! Signal coordinates describe Home data, never process-local authority.
mod branch;
mod conditional_definition;
mod definition;
pub use conditional_definition::{
    DurableSignalConditionalDefinitionOccurrence, SignalConditionalDefinitionOwnerMismatch,
};
mod journal;
mod node;
mod occurrence;
pub use node::DurableSignalNodeIdentity;
mod owner;
pub use branch::{
    DurableSignalBranchIdentity, DurableSignalCellIncarnation, DurableSignalGeneration,
};
pub use definition::{
    SignalBranchAdvanceIdentity, SignalBranchAdvanceReference, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference,
};
pub use journal::SignalDurableJournal;
pub use occurrence::{
    DurableSignalAdmissionIdentity, DurableSignalConditionalOccurrence,
    DurableSignalSnapshotIdentity,
};
pub use owner::{DurableSignalOwnerIdentity, SignalOwnerSlotIdentity};
