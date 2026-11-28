//! Lifecycle coordinates of a selected historical slot generation.
use super::super::visibility::{historical_created_at, historical_retired_at};
use crate::identity::data::VersionId;
use crate::storage::substrate::{HistoricalMetadata, RecordArena, RecordKind};

pub(super) struct HistoricalSlot {
    pub(super) created: VersionId,
    pub(super) retired: Option<VersionId>,
}

impl HistoricalSlot {
    pub(super) fn read<K: RecordKind>(
        arena: &RecordArena<K>,
        slot: usize,
        metadata: &K::Meta,
        version: VersionId,
    ) -> Option<Self> {
        let history = arena.metadata_history_at(slot)?;
        let current = arena.get_slot(slot)?;
        Some(Self {
            created: historical_created_at(
                history,
                metadata,
                current.generation(),
                arena.created_at_for_slot(slot)?,
            ),
            retired: historical_retired_at(metadata.retired_at(), version),
        })
    }
}
