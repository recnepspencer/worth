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
    ) -> usize
    where
        K::Meta: HistoricalMetadata,
    {
        let bound = VersionBound::new(oldest_retained);
        let mut trimmed = 0;
        for slot in K::arena(partition.read()).occupied_slots() {
            let arena = K::arena(partition.read());
            if arena
                .get_slot(slot)
                .is_none_or(|view| view.lifecycle() != RecordLifecycleState::Live)
            {
                continue;
            }
            let eligible = arena.metadata_history_at(slot).is_some_and(|history| {
                history.iter().any(|entry| {
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
                    entry
                        .retired_at()
                        .is_none_or(|retired| bound.retains_retired(retired))
                });
                trimmed += previous - history.len();
            }
        }
        trimmed
    }
}
