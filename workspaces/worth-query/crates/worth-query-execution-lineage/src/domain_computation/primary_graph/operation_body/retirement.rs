//! Prepared retirement custody contains actual native targets, never row IDs as authority.
use super::super::application_contribution::QueryMemoryReservation;
use super::super::WorthQueryInvariantMutationTarget;

pub struct PreparedRetirementBatch<S> {
    pub rule: usize,
    pub targets: Vec<(
        WorthQueryInvariantMutationTarget<S, ()>,
        WorthQueryInvariantMutationTarget<S, ()>,
    )>,
    pub _memory: QueryMemoryReservation,
}

pub struct PreparedBodyRetirements<S> {
    pub batches: Vec<PreparedRetirementBatch<S>>,
    pub _memory: Option<QueryMemoryReservation>,
}
