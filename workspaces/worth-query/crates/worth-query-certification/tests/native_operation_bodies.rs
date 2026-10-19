//! Native body boundaries compile independently of unrelated workflow portfolios.
#[path = "application_graph/document_retention_model.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "This shared reference application also exposes ports used only by the application-graph target."
)]
mod document_retention_model;
#[path = "application_graph/operation_body.rs"]
mod operation_body;
