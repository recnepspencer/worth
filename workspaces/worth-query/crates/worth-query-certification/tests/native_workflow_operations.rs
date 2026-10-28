//! Native owning workflow operations, separate from unrelated body portfolios.
// The concrete runtime's Send/Sync proof traverses im's nested B-tree chunks.
#![recursion_limit = "256"]

#[path = "application_graph/document_retention_model/host/world_resources.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "Shared reference application ports are exercised by independent consumer courts"
)]
pub(crate) mod document_world_resources;
#[path = "application_graph/document_retention_model/operator_identity.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "Shared reference application ports are exercised by independent consumer courts"
)]
pub(crate) mod operator_identity;

#[path = "application_graph/source_only/capability.rs"]
mod capability;
#[path = "application_graph/document_retention_model.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "This same reference application exposes ports used by the other graph and native-body targets."
)]
mod document_retention_model;
#[path = "application_graph/source_only/authorization.rs"]
mod source_authorization;
