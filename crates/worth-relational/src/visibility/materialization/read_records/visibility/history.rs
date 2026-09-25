use crate::identity::data::{VersionBound, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{HistoricalMetadata, SharedColumn, VersionedRelationMetadata};

pub(in super::super) fn visible_metadata<M: HistoricalMetadata + Clone>(
    history: &SharedColumn<M>,
    version_id: VersionId,
) -> Option<&M> {
    let bound = VersionBound::new(version_id);
    let end = history.partition_point(|entry| bound.includes_created(entry.effective_at()));
    (0..end).rev().map(|index| &history[index]).find(|entry| {
        bound.includes_created(entry.effective_at())
            && entry
                .retired_at()
                .is_none_or(|retired| bound.retains_retired(retired))
    })
}

pub(in super::super) fn visible_relation_metadata(
    partition: &PartitionState,
    slot: usize,
    version_id: VersionId,
) -> Option<&VersionedRelationMetadata> {
    let arena = &partition.relation_arena;
    let history = arena.metadata_history_at(slot)?;
    if let Some(metadata) = visible_metadata(history, version_id) {
        return Some(metadata);
    }
    let current = arena.get_slot(slot)?;
    let retired_at = current.retired_at()?;
    if current.lifecycle() != RecordLifecycleState::RetainedDanglingForAudit
        || retired_at > version_id
    {
        return None;
    }
    history.iter().rev().find(|metadata| {
        metadata.generation == current.generation() && metadata.effective_at <= version_id
    })
}

pub(in super::super) fn historical_created_at<M: HistoricalMetadata + Clone>(
    history: &SharedColumn<M>,
    visible: &M,
    current_generation: u32,
    current_created_at: VersionId,
) -> VersionId {
    if visible.generation() == current_generation {
        return current_created_at;
    }
    history
        .iter()
        .find(|entry| entry.generation() == visible.generation())
        .map(HistoricalMetadata::effective_at)
        .unwrap_or_else(|| visible.effective_at())
}
