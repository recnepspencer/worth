#[allow(ambiguous_glob_reexports, hidden_glob_reexports, reason = "Preserve authored fixture scopes while concrete uses retain compiler checking.")]
#[path = "host/mod.rs"]
pub mod host;
pub(crate) use worth_query_execution::facade::primary_graph;
