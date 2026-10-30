//! Immutable vocabulary exchanged by durable owner ports.

mod home_binding;
mod home_identity;
mod home_incarnation;
mod owner_family;
mod retention_class;
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
