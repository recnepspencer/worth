//! Immutable vocabulary exchanged by durable owner ports.

mod home_binding;
mod world_identity;
mod world_journal;
pub use world_identity::*;
pub use world_journal::WorldDurableJournal;
mod composite_attempt_association;
pub use composite_attempt_association::{
    CompositeAttemptAssociation, CompositeAttemptAssociationOwnerMismatch,
};
mod home_identity;
mod home_incarnation;
mod owner_family;
mod query_output_settlement;
mod retention_class;
mod state_family;
pub use query_output_settlement::DurableQueryOutputSettlement;
pub use state_family::DurableStateFamily;
#[cfg(test)]
mod tests;

mod identity_seal {
    pub trait Sealed {}
}

/// Data whose identity is meaningful after its owning home is reopened.
///
/// This marker grants no runtime authority. Capabilities and process-local
/// identities cannot implement its private predecessor.
///
/// ```compile_fail,E0277
/// use worth_foundational::facade::DurableIdentity;
/// struct ProcessRuntimeOrdinal(u64);
/// impl DurableIdentity for ProcessRuntimeOrdinal {}
/// ```
pub trait DurableIdentity: identity_seal::Sealed {}

pub use home_binding::{
    ApplicationHomeBinding, ApplicationHomeBindingDescriptionDenial, HomeProgramBinding,
};
pub use home_identity::{HomeIdentity, HomeIdentityDenial};
pub use home_incarnation::{HomeIncarnation, HomeIncarnationExhausted};
pub use owner_family::{DurableOwnerFamily, UnsupportedDurableOwnerTag};
pub use retention_class::{DurableArtifactRetention, RetentionClass};
