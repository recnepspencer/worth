//! Framework-owned traversal completes before any candidate result is returned.
mod exhaustive;
pub(in crate::domain_computation::primary_graph) use exhaustive::visit;
