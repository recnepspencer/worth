//! Native owning workflow operations, separate from unrelated body portfolios.
#[path = "application_graph/source_only/capability.rs"]
mod capability;
#[path = "application_graph/document_retention_model.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "This same reference application exposes ports used by the other graph and native-body targets."
)]
mod document_retention_model;
#[path = "support/execution_policy.rs"]
mod test_request_policy;
