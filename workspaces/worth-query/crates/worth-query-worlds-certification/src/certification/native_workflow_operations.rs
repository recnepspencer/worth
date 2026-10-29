//! Shared fixtures for native owning workflow operation courts.
// The concrete runtime's Send/Sync proof traverses im's nested B-tree chunks.
pub use crate::facade::certification::document_world_resources;
pub use crate::facade::certification::operator_identity;

#[path = "application_graph/source_only/capability.rs"]
pub mod capability;
pub use crate::facade::certification::document_retention_model;
#[path = "application_graph/source_only/authorization.rs"]
pub mod source_authorization;
