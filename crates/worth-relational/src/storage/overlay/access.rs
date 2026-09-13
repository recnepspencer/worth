use std::collections::BTreeMap;

use crate::identity::data::PartitionId;

use super::PartitionState;

pub(crate) trait PartitionAccess: Sync {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState>;
    fn partition_ids(&self) -> Vec<PartitionId>;

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        None
    }

    fn touched_entity_slots(&self, _partition_id: PartitionId) -> Option<Vec<usize>> {
        None
    }

    fn touched_relation_slots(&self, _partition_id: PartitionId) -> Option<Vec<usize>> {
        None
    }

    fn entity_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.touched_entity_slots(partition_id)
            .is_some_and(|slots| slots.contains(&slot))
    }

    fn relation_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.touched_relation_slots(partition_id)
            .is_some_and(|slots| slots.contains(&slot))
    }

    fn base_partition(&self, _partition_id: PartitionId) -> Option<&PartitionState> {
        None
    }
}

impl PartitionAccess for BTreeMap<PartitionId, PartitionState> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.get(&partition_id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.keys().copied().collect()
    }
}

#[derive(Clone)]
pub(crate) struct OverlayStateView<'a, S> {
    base_partitions: &'a dyn PartitionAccess,
    staged: &'a S,
}

impl<'a, S> OverlayStateView<'a, S> {
    pub(crate) fn new(base_partitions: &'a dyn PartitionAccess, staged: &'a S) -> Self {
        Self {
            base_partitions,
            staged,
        }
    }

    pub(crate) fn base_partition_access(&self) -> &'a dyn PartitionAccess {
        self.base_partitions
    }
}

impl<S: PartitionAccess> PartitionAccess for OverlayStateView<'_, S> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.staged
            .get_partition(partition_id)
            .or_else(|| self.base_partitions.get_partition(partition_id))
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        let base_ids = self.base_partitions.partition_ids();
        let staged_ids = self.staged.partition_ids();
        debug_assert!(partition_ids_are_canonical(staged_ids.iter().copied()));

        let mut merged = Vec::with_capacity(base_ids.len() + staged_ids.len());
        let mut base_iter = base_ids.into_iter().peekable();
        let mut staged_iter = staged_ids.into_iter().peekable();

        loop {
            match (base_iter.peek().copied(), staged_iter.peek().copied()) {
                (Some(base), Some(staged)) => match base.cmp(&staged) {
                    std::cmp::Ordering::Less => {
                        merged.push(base);
                        base_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        merged.push(staged);
                        staged_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        merged.push(base);
                        base_iter.next();
                        staged_iter.next();
                    }
                },
                (Some(base), None) => {
                    merged.push(base);
                    base_iter.next();
                    merged.extend(base_iter);
                    break;
                }
                (None, Some(staged)) => {
                    merged.push(staged);
                    staged_iter.next();
                    merged.extend(staged_iter);
                    break;
                }
                (None, None) => break,
            }
        }

        merged
    }

    fn touched_entity_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.staged.touched_entity_slots(partition_id)
    }

    fn touched_relation_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.staged.touched_relation_slots(partition_id)
    }

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        self.staged.touched_partition_ids()
    }

    fn entity_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.staged.entity_slot_is_touched(partition_id, slot)
    }

    fn relation_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.staged.relation_slot_is_touched(partition_id, slot)
    }

    fn base_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.base_partitions.get_partition(partition_id)
    }
}

fn partition_ids_are_canonical(partition_ids: impl IntoIterator<Item = PartitionId>) -> bool {
    let mut partition_ids = partition_ids.into_iter();
    let Some(mut previous) = partition_ids.next() else {
        return true;
    };
    for current in partition_ids {
        if previous >= current {
            return false;
        }
        previous = current;
    }
    true
}
