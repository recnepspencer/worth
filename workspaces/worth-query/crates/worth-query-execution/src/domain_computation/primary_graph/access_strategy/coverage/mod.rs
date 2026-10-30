//! Framework-owned traversal completes before any candidate result is returned.
mod traversal;
pub(in crate::domain_computation::primary_graph) use traversal::visit;
