//! Home-scoped World data. These values describe identity, never grant authority.
mod branch;
mod coordinate;
mod occurrence;
mod owner;
mod reference;
#[cfg(test)]
mod tests;
pub use branch::{
    DurableProductBranchIdentity, DurableProductBranchNameDenial, DurableProductReferenceGeneration,
};
pub use coordinate::WorldOwnerCoordinate;
pub use occurrence::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIncarnation, DurableProductUnpublishedOwnerEffectsIdentity,
    DurableWorldBootstrapAttemptIdentity,
};
pub use owner::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
pub use reference::{DurableProductBranchReference, DurableProductReferenceOwnerMismatch};
