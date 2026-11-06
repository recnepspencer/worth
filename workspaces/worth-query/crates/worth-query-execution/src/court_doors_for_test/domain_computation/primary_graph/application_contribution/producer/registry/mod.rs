//! Doors under `domain_computation::primary_graph::application_contribution::producer::registry`.
#[path = "checkpoint/mod.rs"]
pub mod checkpoint;
#[path = "operation_binding_uniqueness/mod.rs"]
pub mod operation_binding_uniqueness;
#[path = "output_family_inventory/mod.rs"]
pub mod output_family_inventory;
#[path = "scope_ownership/mod.rs"]
pub mod scope_ownership;
pub use crate::domain_computation::primary_graph::application_contribution::producer::registry::DeclaredProducerBinding;
pub use crate::domain_computation::primary_graph::application_contribution::producer::registry::InstalledProducerProvider;
