pub(crate) mod candidate_source;
pub(crate) mod custom_registry;
pub(crate) mod custom_rule;
pub mod data;
pub mod engine;
pub mod execution;
mod invariant_access;
pub(crate) mod invariant_authority;
mod preparation_invariant_authority;
mod proposed_state;
pub mod reduction;

pub use custom_registry::DuplicateCustomInvariantRegistration;
pub(crate) use custom_registry::{
    CustomInvariantCompilationDenial, FrozenCustomInvariantRegistry, SourcePreparationInventory,
};
pub(crate) use custom_rule::StructuralPresenceDenial;
pub(crate) use custom_rule::{CustomInvariantWorkMeter, CustomPreparationBudget};
pub use invariant_access::InvariantAccess;
