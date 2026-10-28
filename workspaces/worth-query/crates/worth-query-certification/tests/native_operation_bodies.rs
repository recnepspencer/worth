//! Native body boundaries compile independently of unrelated workflow portfolios.
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

#[path = "application_graph/document_retention_model.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "This shared reference application also exposes ports used only by the application-graph target."
)]
mod document_retention_model;
#[path = "application_graph/operation_body.rs"]
mod operation_body;
