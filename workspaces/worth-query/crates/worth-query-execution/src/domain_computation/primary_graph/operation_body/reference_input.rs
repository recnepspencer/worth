//! Retained existing endpoints; candidate assembly never resolves the input again.
use {crate::domain_computation::primary_graph::request_execution::QueryMemoryReservation, super::super::WorthQueryInvariantMutationTarget};
pub(in crate::domain_computation::primary_graph) struct PreparedBodyReference<S> {
    pub current: Option<WorthQueryInvariantMutationTarget<S, ()>>,
    pub selected: Option<WorthQueryInvariantMutationTarget<S, ()>>,
}
pub(in crate::domain_computation::primary_graph) struct PreparedBodyReferences<S> {
    pub targets: Vec<PreparedBodyReference<S>>,
    pub _memory: Option<QueryMemoryReservation>,
}
