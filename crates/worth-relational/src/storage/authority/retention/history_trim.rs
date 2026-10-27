//! Destructive live-history maintenance requires the pass's accepted Full entry.
use crate::identity::data::{VersionBound, VersionId};
use crate::runtime::ResidentPartitionEntry;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{HistoricalMetadata, RecordKind};

use super::super::StorageAuthority;

impl StorageAuthority<'_> {
    pub(crate) fn trim_full_live_history<K: RecordKind>(
        &self,
        partition: &mut ResidentPartitionEntry<'_>,
        oldest_retained: VersionId,
    ) -> Result<usize, crate::execution::RelationalExecutionDenialCause>
    where
        K::Meta: HistoricalMetadata,
    {
        let bound = VersionBound::new(oldest_retained);
        let mut trimmed = 0;
        let mut slots_visited = 0;
        let mut entries_visited = 0;
        for slot in K::arena(partition.read()?).occupied_slots() {
            slots_visited += 1;
            let arena = K::arena(partition.read()?);
            if arena
                .get_slot(slot)
                .is_none_or(|view| view.lifecycle() != RecordLifecycleState::Live)
            {
                continue;
            }
            let eligible = arena.metadata_history_at(slot).is_some_and(|history| {
                history.iter().any(|entry| {
                    entries_visited += 1;
                    entry
                        .retired_at()
                        .is_some_and(|retired| !bound.retains_retired(retired))
                })
            });
            if !eligible {
                continue;
            }
            let arena = K::arena_mut(partition.edit());
            if let Some(history) = arena.metadata_history_at_mut(slot) {
                let previous = history.len();
                history.retain(|entry| {
                    entries_visited += 1;
                    entry
                        .retired_at()
                        .is_none_or(|retired| bound.retains_retired(retired))
                });
                trimmed += previous - history.len();
            }
        }
        self.runtime.services.instrumentation.count(|counters| {
            counters.live_history_slots_visited += slots_visited;
            counters.live_history_entries_visited += entries_visited;
        });
        Ok(trimmed)
    }
}
