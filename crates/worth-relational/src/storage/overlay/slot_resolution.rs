//! One sparse overlay/base row selection rule for native reads and preparation.
use super::CurrentPartitionAccess;
use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRegionHeadView;

#[derive(Clone, Copy)]
pub(crate) enum PartitionSource {
    Visible,
    Base,
}

fn entity_uses_visible(state: &dyn CurrentPartitionAccess, id: PartitionId, slot: usize) -> bool {
    state.entity_slot_is_touched(id, slot) || !state.has_touched_entity_slots(id)
}

fn relation_uses_visible(
    state: &dyn CurrentPartitionAccess,
    id: PartitionId,
    slot: usize,
    slot_present: bool,
    sparse: bool,
) -> bool {
    state.relation_slot_is_touched(id, slot)
        || (!state.has_touched_relation_slots(id) && slot_present)
        || (slot_present && !sparse)
}

fn select<T>(
    visible: T,
    use_visible: bool,
    base: impl FnOnce() -> Option<T>,
) -> (PartitionSource, T) {
    if !use_visible && let Some(base) = base() {
        return (PartitionSource::Base, base);
    }
    (PartitionSource::Visible, visible)
}

pub(crate) fn current_entity_partition_source_for_slot(
    state: &dyn CurrentPartitionAccess,
    partition_id: PartitionId,
    slot: usize,
) -> Option<(PartitionSource, RelationalRegionHeadView<'_>)> {
    let partition = state.get_current_partition(partition_id)?;
    Some(select(
        partition,
        entity_uses_visible(state, partition_id, slot),
        || state.base_current_partition(partition_id),
    ))
}

pub(crate) fn current_relation_partition_source_for_slot(
    state: &dyn CurrentPartitionAccess,
    partition_id: PartitionId,
    slot: usize,
) -> Option<(PartitionSource, RelationalRegionHeadView<'_>)> {
    let partition = state.get_current_partition(partition_id)?;
    let use_visible = relation_uses_visible(
        state,
        partition_id,
        slot,
        partition.relation_arena().get_slot(slot).is_some(),
        partition.relation_overlay_is_sparse(),
    );
    Some(select(partition, use_visible, || {
        state.base_current_partition(partition_id)
    }))
}
