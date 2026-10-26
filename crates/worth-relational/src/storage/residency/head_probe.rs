//! Head discovery discards every provisional result before loading its misses.
use super::discovery_policy::{discover, MissFootprint, ReadMiss};
use super::RelationalRegionHeadView;
use crate::identity::data::PartitionId;
use crate::storage::overlay::CurrentPartitionAccess;
use std::sync::Mutex;

pub(crate) enum PartitionHeadProbe<'view> {
    Absent,
    Cold,
    Resident(RelationalRegionHeadView<'view>),
}

struct HeadReadRound<'source, 'round> {
    source: &'source dyn CurrentPartitionAccess,
    source_index: usize,
    misses: &'round Mutex<MissFootprint>,
}

impl<'source> HeadReadRound<'source, '_> {
    fn read_partition(
        &self,
        source: &'source dyn CurrentPartitionAccess,
        id: PartitionId,
        base: bool,
    ) -> Option<RelationalRegionHeadView<'source>> {
        match source.probe_head_partition(id) {
            PartitionHeadProbe::Resident(partition) => Some(partition),
            PartitionHeadProbe::Absent => None,
            PartitionHeadProbe::Cold => {
                self.misses
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .record(ReadMiss {
                        source_index: self.source_index,
                        base,
                        partition: id,
                    });
                None
            }
        }
    }
}

impl CurrentPartitionAccess for HeadReadRound<'_, '_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.read_partition(self.source, id, false)
    }
    fn base_current_source(&self) -> Option<&dyn CurrentPartitionAccess> {
        self.source.base_current_source()
    }
    fn base_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.read_partition(self.source.base_current_source()?, id, true)
    }
    fn has_read_misses(&self) -> bool {
        self.misses
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .first()
            .is_some()
    }

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        self.source.touched_partition_ids()
    }

    fn touched_entity_slots(&self, id: PartitionId) -> Option<Vec<usize>> {
        self.source.touched_entity_slots(id)
    }

    fn touched_relation_slots(&self, id: PartitionId) -> Option<Vec<usize>> {
        self.source.touched_relation_slots(id)
    }

    fn has_touched_entity_slots(&self, id: PartitionId) -> bool {
        self.source.has_touched_entity_slots(id)
    }

    fn has_touched_relation_slots(&self, id: PartitionId) -> bool {
        self.source.has_touched_relation_slots(id)
    }

    fn entity_slot_is_touched(&self, id: PartitionId, slot: usize) -> bool {
        self.source.entity_slot_is_touched(id, slot)
    }

    fn relation_slot_is_touched(&self, id: PartitionId, slot: usize) -> bool {
        self.source.relation_slot_is_touched(id, slot)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.source.partition_ids()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        self.source.partition_ids_iter()
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        self.source.partition_ids_with_memory(memory)
    }
}

/// The source retains every admitted lease throughout every round. A failed
/// round's owned result is dropped before I/O, including a provisional denial.
/// At most one round per newly discovered region is possible. After the policy
/// ceiling, demand the entire source, so even adversarial traversals converge.
pub(crate) fn probe_head<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &dyn CurrentPartitionAccess,
    mut read: impl FnMut(&dyn CurrentPartitionAccess) -> T,
) -> Result<T, super::RelationalResidencyDemandFailure> {
    probe_head_sources(execution, [source], |[view]| read(view))
}

/// Related views share one miss ledger, including overlay before-images.
/// The fixed source array and inline miss footprint need no heap allocation.
pub(crate) fn probe_head_sources<const N: usize, T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    sources: [&dyn CurrentPartitionAccess; N],
    mut read: impl FnMut([&dyn CurrentPartitionAccess; N]) -> T,
) -> Result<T, super::RelationalResidencyDemandFailure> {
    discover(
        execution,
        |misses| {
            let views: [_; N] = std::array::from_fn(|source_index| HeadReadRound {
                source: sources[source_index],
                source_index,
                misses,
            });
            read(std::array::from_fn(|index| {
                &views[index] as &dyn CurrentPartitionAccess
            }))
        },
        |miss| {
            let source = sources[miss.source_index];
            let source = if miss.base {
                source
                    .base_current_source()
                    .expect("a recorded base miss has a base source")
            } else {
                source
            };
            source.demand_head_partition(execution, miss.partition)
        },
        || {
            for source in sources {
                for id in source.partition_ids_iter() {
                    source.demand_head_partition(execution, id)?;
                }
                if let Some(base) = source.base_current_source() {
                    for id in base.partition_ids_iter() {
                        base.demand_head_partition(execution, id)?;
                    }
                }
            }
            Ok(())
        },
    )
}

#[cfg(test)]
#[test]
fn current_sources_cannot_implement_the_full_history_source_contract() {
    trait WithoutFull<A> {
        fn witness() {}
    }
    impl<T: ?Sized> WithoutFull<()> for T {}
    impl<T: ?Sized + crate::storage::overlay::PartitionAccess> WithoutFull<u8> for T {}
    let _ = <HeadReadRound<'_, '_> as WithoutFull<_>>::witness;
    let _ = <crate::storage::overlay::WorkingState as WithoutFull<_>>::witness;
    let _ = <crate::storage::overlay::OverlayStateView<'_, crate::storage::overlay::WorkingState> as WithoutFull<_>>::witness;
    let _ = <crate::branch::SelectedRelationalBranchRead<'_> as WithoutFull<_>>::witness;
}
