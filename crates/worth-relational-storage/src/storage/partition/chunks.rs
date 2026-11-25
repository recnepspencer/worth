//! Summarize visible, retained and reclaimable records by physical arena chunk.
use std::collections::BTreeMap;

use crate::identity::data::{VersionBound, VersionId};
use crate::storage::data::{ChunkVisibilitySummary, RecordLifecycleState};
use crate::storage::substrate::{HistoricalMetadata, RecordArena, RecordKind};

pub fn summarize_arena_chunks<K: RecordKind>(
    arena: &RecordArena<K>,
    version_id: VersionId,
    current_version: VersionId,
    chunk_size: usize,
) -> Vec<ChunkVisibilitySummary>
where
    K::Meta: HistoricalMetadata,
{
    if chunk_size == 0 {
        return Vec::new();
    }
    let occupied = arena.occupied_slots();
    let Some(logical_end) = occupied.last().and_then(|slot| slot.checked_add(1)) else {
        return Vec::new();
    };
    let mut chunks = BTreeMap::<usize, ChunkAccumulator>::new();
    for slot in occupied {
        let visible = if version_id == current_version {
            arena.live_bitset.count_ones_in_range(slot, slot + 1) == 1
        } else {
            arena
                .metadata_history_at(slot)
                .is_some_and(|history| visible_at_version(history, version_id))
        };
        chunks
            .entry(slot_chunk_index(slot, chunk_size))
            .or_default()
            .observe(
                visible,
                arena.created_at_for_slot(slot),
                arena.retired_at_for_slot(slot),
                arena.get_slot(slot).map(|view| view.lifecycle()),
            );
    }
    chunks
        .into_iter()
        .map(|(chunk_index, values)| {
            let slot_start = chunk_index.saturating_mul(chunk_size);
            ChunkVisibilitySummary {
                chunk_index,
                slot_start,
                slot_len: logical_end.saturating_sub(slot_start).min(chunk_size),
                visible_records: values.visible_records,
                retained_records: values.retained_records,
                reclaimable_records: values.reclaimable_records,
                earliest_created_at: values.earliest_created_at,
                latest_retired_at: values.latest_retired_at,
            }
        })
        .collect()
}

fn visible_at_version<M: HistoricalMetadata + Clone>(
    history: &crate::storage::substrate::SharedColumn<
        crate::storage::substrate::StorageAllocation<M>,
    >,
    version_id: VersionId,
) -> bool {
    let bound = VersionBound::new(version_id);
    let end = history.partition_point(|entry| bound.includes_created(entry.effective_at()));
    (0..end).rev().map(|index| &history[index]).any(|entry| {
        entry
            .retired_at()
            .is_none_or(|retired| bound.retains_retired(retired))
    })
}

#[derive(Default)]
struct ChunkAccumulator {
    visible_records: usize,
    retained_records: usize,
    reclaimable_records: usize,
    earliest_created_at: Option<VersionId>,
    latest_retired_at: Option<VersionId>,
}

impl ChunkAccumulator {
    fn observe(
        &mut self,
        visible: bool,
        created_at: Option<VersionId>,
        retired_at: Option<VersionId>,
        lifecycle: Option<RecordLifecycleState>,
    ) {
        self.visible_records += usize::from(visible);
        if let Some(created_at) = created_at {
            self.earliest_created_at = Some(
                self.earliest_created_at
                    .map_or(created_at, |current| current.min(created_at)),
            );
        }
        if let Some(retired_at) = retired_at {
            self.retained_records += 1;
            self.latest_retired_at = Some(
                self.latest_retired_at
                    .map_or(retired_at, |current| current.max(retired_at)),
            );
        }
        self.reclaimable_records +=
            usize::from(lifecycle == Some(RecordLifecycleState::Reclaimable));
    }
}

pub fn slot_chunk_index(slot: usize, chunk_size: usize) -> usize {
    // A zero-sized diagnostic chunk places every slot in bucket zero.
    slot.checked_div(chunk_size).unwrap_or(0)
}
