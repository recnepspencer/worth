//! Shared fixtures for native owning workflow operation courts.
// The concrete runtime's Send/Sync proof traverses im's nested B-tree chunks.
pub use worth_query_testkit_document_retention::facade::document_world_resources;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity;

#[path = "application_graph/source_only/capability.rs"]
pub mod capability;
pub use worth_query_testkit_document_retention::facade::document_retention_model;
#[path = "application_graph/source_only/authorization.rs"]
pub mod source_authorization;
