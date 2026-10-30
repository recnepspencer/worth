//! The document-retention world and its schema-neutral resources.
#[path = "certification/application_graph/cell_history.rs"]
pub mod cell_history;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Preserve authored lexical fixture imports; actual ambiguous uses remain compiler errors."
)]
#[path = "certification/application_graph/document_retention_model.rs"]
pub mod document_retention_model;
#[path = "certification/application_graph/document_retention_model/host/world_resources.rs"]
pub mod document_world_resources;
