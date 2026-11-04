//! Existing anchors are observed once and carried into candidate assembly.
use {crate::domain_computation::primary_graph::request_execution::QueryMemoryReservation, super::super::WorthQueryInvariantMutationTarget};
pub(in crate::domain_computation::primary_graph) struct PreparedBodyPathLinks<S> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<WorthQueryInvariantMutationTarget<S, ()>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
