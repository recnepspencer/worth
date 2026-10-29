//! The host world-family facade.
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts use concrete definitions through these scopes."
)]
#[path = "host/mod.rs"]
pub mod host;
// The shared owner-unit oracle keeps its original facade path.
pub(crate) use worth_query_execution::facade::primary_graph;
