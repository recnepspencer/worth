//! Retain only the actual business interpreter's cohort-relevant output.
use crate::authority::mutation::AdjacencyDelta;
use crate::transactions::data::RecordRef;

pub(super) struct AggregateChanges {
    pub(super) has_entity_deletions: bool,
    pub(super) adjacency: Vec<AdjacencyDelta>,
    pub(super) records: Vec<RecordRef>,
}

impl AggregateChanges {
    pub(super) fn from_applied(
        records: &[RecordRef],
        adjacency: &[AdjacencyDelta],
        has_entity_deletions: bool,
    ) -> Self {
        Self {
            has_entity_deletions,
            adjacency: adjacency.to_vec(),
            records: records.to_vec(),
        }
    }

    pub(super) fn retained_bytes(&self) -> u64 {
        (self.adjacency.capacity() * std::mem::size_of::<AdjacencyDelta>()
            + self.records.capacity() * std::mem::size_of::<RecordRef>()) as u64
    }
}
