//! Home-scoped World data. These values describe identity, never grant authority.
mod branch;
mod occurrence;
mod owner;
#[cfg(test)]
mod tests;
pub use branch::{
    DurableProductBranchIdentity, DurableProductBranchNameDenial, DurableProductReferenceGeneration,
};
pub use occurrence::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIncarnation, DurableProductUnpublishedOwnerEffectsIdentity,
    DurableWorldBootstrapAttemptIdentity,
};
pub use owner::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
