//! Decision-owned selection preimages, never caller-supplied targets.
use super::WorthQueryInvariantMutationTarget;
use crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation;
pub(in crate::domain_computation::primary_graph) struct PreparedBodyReplacements<Schema> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<Option<WorthQueryInvariantMutationTarget<Schema, ()>>>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}
