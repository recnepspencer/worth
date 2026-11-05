//! Moved whole from `domain_computation::primary_graph::operation_body::path_value`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use crate::domain_computation::request_execution::QueryMemoryReservation;

pub(in crate::domain_computation::primary_graph) struct PreparedBodyPathWrites<S> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<WorthQueryInvariantMutationTarget<S, ()>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
