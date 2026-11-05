//! Moved whole from `domain_computation::primary_graph::operation_body::reference_input`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use crate::domain_computation::request_execution::QueryMemoryReservation;

pub(in crate::domain_computation::primary_graph) struct PreparedBodyReference<S> {
    pub current: Option<WorthQueryInvariantMutationTarget<S, ()>>,
    pub selected: Option<WorthQueryInvariantMutationTarget<S, ()>>,
}

pub(in crate::domain_computation::primary_graph) struct PreparedBodyReferences<S> {
    pub targets: Vec<PreparedBodyReference<S>>,
    pub _memory: Option<QueryMemoryReservation>,
}
