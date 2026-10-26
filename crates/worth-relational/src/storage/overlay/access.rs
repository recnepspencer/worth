use std::collections::BTreeMap;

use crate::identity::data::PartitionId;

use super::PartitionState;
mod memory;
#[cfg(test)]
use memory::partition_id_bytes;
pub(crate) use memory::{collect_partition_ids, PartitionIdMemory, PartitionIdsDenial};

pub(crate) trait PartitionAccess: super::CurrentPartitionAccess {
    fn base_source(&self) -> Option<&dyn PartitionAccess> {
        None
    }

    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState>;

    fn probe_full_partition(
        &self,
        partition_id: PartitionId,
    ) -> super::super::residency::PartitionProbe<'_> {
        match self.get_partition(partition_id) {
            Some(partition) => super::super::residency::PartitionProbe::Resident(partition),
            None => super::super::residency::PartitionProbe::Absent,
        }
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _partition_id: PartitionId,
    ) -> Result<(), super::super::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(super::super::residency::RelationalResidencyDemandFailure::Request)?;
        Ok(())
    }

    fn base_partition(&self, _partition_id: PartitionId) -> Option<&PartitionState> {
        None
    }
}

impl PartitionAccess for BTreeMap<PartitionId, PartitionState> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.get(&partition_id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for BTreeMap<PartitionId, PartitionState> {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        crate::storage::overlay::PartitionAccess::get_partition(self, id)
            .map(crate::storage::residency::RelationalRegionHeadView::from_owned_partition)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.keys().copied().collect()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        collect_partition_ids(self.len(), self.keys().copied(), memory)
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.keys().copied())
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
    fn base_source(&self) -> Option<&dyn PartitionAccess> {
        Some(self.base_partitions)
    }

    fn probe_full_partition(&self, id: PartitionId) -> super::super::residency::PartitionProbe<'_> {
        match self.staged.probe_full_partition(id) {
            super::super::residency::PartitionProbe::Absent => {
                self.base_partitions.probe_full_partition(id)
            }
            probe => probe,
        }
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), super::super::residency::RelationalResidencyDemandFailure> {
        match self.staged.probe_full_partition(id) {
            super::super::residency::PartitionProbe::Absent => {
                self.base_partitions.demand_full_partition(execution, id)
            }
            _ => self.staged.demand_full_partition(execution, id),
        }
    }

    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.staged
            .get_partition(partition_id)
            .or_else(|| self.base_partitions.get_partition(partition_id))
    }

    fn base_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.base_partitions.get_partition(partition_id)
    }
}

impl<S: PartitionAccess> crate::storage::overlay::CurrentPartitionAccess
    for OverlayStateView<'_, S>
{
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.staged
            .get_current_partition(id)
            .or_else(|| self.base_partitions.get_current_partition(id))
    }
    fn base_current_source(&self) -> Option<&dyn crate::storage::overlay::CurrentPartitionAccess> {
        Some(self.base_partitions)
    }
    fn base_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.base_partitions.get_current_partition(id)
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        match self.staged.probe_head_partition(id) {
            crate::storage::residency::PartitionHeadProbe::Absent => {
                self.base_partitions.probe_head_partition(id)
            }
            probe => probe,
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        match self.staged.probe_head_partition(id) {
            crate::storage::residency::PartitionHeadProbe::Absent => {
                self.base_partitions.demand_head_partition(execution, id)
            }
            _ => self.staged.demand_head_partition(execution, id),
        }
    }

    fn has_read_misses(&self) -> bool {
        self.staged.has_read_misses() || self.base_partitions.has_read_misses()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        let base = self.base_partitions.partition_ids_with_memory(memory)?;
        let staged = match self.staged.partition_ids_with_memory(memory) {
            Ok(ids) => ids,
            Err(error) => {
                memory::release_ids(base, memory);
                return Err(error);
            }
        };
        debug_assert!(partition_ids_are_canonical(staged.iter().copied()));
        memory::merge_partition_ids(base, staged, memory)
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

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        let mut base = self.base_partitions.partition_ids_iter().peekable();
        let mut staged = self.staged.partition_ids_iter().peekable();
        Box::new(std::iter::from_fn(move || {
            match (base.peek().copied(), staged.peek().copied()) {
                (Some(left), Some(right)) => match left.cmp(&right) {
                    std::cmp::Ordering::Less => base.next(),
                    std::cmp::Ordering::Greater => staged.next(),
                    std::cmp::Ordering::Equal => {
                        staged.next();
                        base.next()
                    }
                },
                (Some(_), None) => base.next(),
                (None, Some(_)) => staged.next(),
                (None, None) => None,
            }
        }))
    }

    fn touched_entity_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.staged.touched_entity_slots(partition_id)
    }

    fn touched_relation_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.staged.touched_relation_slots(partition_id)
    }

    fn has_touched_entity_slots(&self, partition_id: PartitionId) -> bool {
        self.staged.has_touched_entity_slots(partition_id)
    }

    fn has_touched_relation_slots(&self, partition_id: PartitionId) -> bool {
        self.staged.has_touched_relation_slots(partition_id)
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

#[cfg(test)]
mod tests;
