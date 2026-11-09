pub(crate) mod candidate_source;
mod custom_registry;
mod custom_rule;
pub mod data;
pub mod engine;
pub mod execution;
mod invariant_access;
mod invariant_authority;
mod preparation_invariant_authority;
pub mod reduction;

pub use custom_registry::DuplicateCustomInvariantRegistration;
pub(crate) use custom_registry::{
    CustomInvariantCompilationDenial, CustomInvariantCompilationFunding,
    FrozenCustomInvariantRegistry, SourcePreparationInventory,
};
pub(crate) use custom_rule::StructuralPresenceDenial;
pub(crate) use custom_rule::{CustomInvariantWorkMeter, CustomPreparationBudget};
pub use invariant_access::InvariantAccess;
