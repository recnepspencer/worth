//! The actual business interpreter's cohort-relevant output, moved without copying.
use crate::authority::mutation::{AdjacencyDelta, MutationEffect};
use crate::transactions::data::RecordRef;

pub(super) struct AggregateChanges {
    pub(super) adjacency: Vec<AdjacencyDelta>,
    pub(super) records: Vec<RecordRef>,
}

impl AggregateChanges {
    pub(super) fn from_applied(effect: MutationEffect) -> Self {
        Self {
            adjacency: effect.adjacency.deltas,
            records: effect.publication.changed_records,
        }
    }

    pub(super) fn retained_bytes(&self) -> u64 {
        (self.adjacency.capacity() * std::mem::size_of::<AdjacencyDelta>()
            + self.records.capacity() * std::mem::size_of::<RecordRef>()) as u64
    }
}
