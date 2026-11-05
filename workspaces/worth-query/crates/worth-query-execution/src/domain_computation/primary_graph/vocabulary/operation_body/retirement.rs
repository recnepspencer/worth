//! Moved whole from `domain_computation::primary_graph::operation_body::retirement`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use crate::domain_computation::request_execution::QueryMemoryReservation;

pub(in crate::domain_computation::primary_graph) struct PreparedRetirementBatch<S> {
    pub(in crate::domain_computation::primary_graph) rule: usize,
    pub(in crate::domain_computation::primary_graph) targets: Vec<(
        WorthQueryInvariantMutationTarget<S, ()>,
        WorthQueryInvariantMutationTarget<S, ()>,
    )>,
    pub(in crate::domain_computation::primary_graph) _memory: QueryMemoryReservation,
}

pub(in crate::domain_computation::primary_graph) struct PreparedBodyRetirements<S> {
    pub(in crate::domain_computation::primary_graph) batches: Vec<PreparedRetirementBatch<S>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
