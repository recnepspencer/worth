//! Carried completeness of the history physically present in a resident region.
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{RecordHistoryPrefix, SharedColumn};

#[derive(Debug, Clone, Copy)]
pub(super) struct HistoryPresence {
    omitted: usize,
}

/// Transient edit basis, not a linked history of earlier region versions.
pub(super) struct HistoryPresenceBasis {
    presence: HistoryPresence,
    entities: SharedColumn<Option<RecordHistoryPrefix>>,
    relations: SharedColumn<Option<RecordHistoryPrefix>>,
}

impl HistoryPresence {
    /// Full custody already proves every historical row is materialized.
    pub(super) fn complete() -> Self {
        Self { omitted: 0 }
    }
    /// cost-lane: cold reconstruction from authenticated prefix openings.
    pub(super) fn from_cold(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        partition: &PartitionState,
    ) -> Self {
        Self {
            omitted: count(&partition.entity_arena.history_prefixes)
                + count(&partition.relation_arena.history_prefixes),
        }
    }
    pub(super) fn is_complete(self) -> bool {
        self.omitted == 0
    }
    pub(super) fn basis(self, partition: &PartitionState) -> HistoryPresenceBasis {
        HistoryPresenceBasis {
            presence: self,
            entities: partition.entity_arena.history_prefixes.clone(),
            relations: partition.relation_arena.history_prefixes.clone(),
        }
    }
    pub(super) fn updated(self, previous: &PartitionState, current: &PartitionState) -> Self {
        self.basis(previous).updated(current)
    }
}
impl HistoryPresenceBasis {
    pub(super) fn updated(&self, current: &PartitionState) -> HistoryPresence {
        let omitted = adjust(
            self.presence.omitted,
            &self.entities,
            &current.entity_arena.history_prefixes,
        );
        HistoryPresence {
            omitted: adjust(
                omitted,
                &self.relations,
                &current.relation_arena.history_prefixes,
            ),
        }
    }
}

fn present<T>(value: Option<&Option<T>>) -> usize {
    usize::from(value.is_some_and(Option::is_some))
}
fn count<T: Clone>(column: &SharedColumn<Option<T>>) -> usize {
    column.iter().map(|value| present(Some(value))).sum()
}
fn adjust<T: Clone>(
    previous_count: usize,
    previous: &SharedColumn<Option<T>>,
    current: &SharedColumn<Option<T>>,
) -> usize {
    let (mut removed, mut added) = (0, 0);
    current.visit_changed_values(previous, &mut |_, new, old| {
        removed += present(old);
        added += present(new);
    });
    // Proof: removed visits a subset of the counted previous column, and
    // added visits a subset of the current addressable column. Both immutable
    // columns come from the same region's owned before/after storage edit.
    previous_count - removed + added
}

#[cfg(test)]
mod tests;
