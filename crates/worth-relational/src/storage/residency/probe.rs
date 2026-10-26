//! Full-history rounds over an explicit source. Only a miss-free round escapes.
use super::discovery_policy::{discover, MissFootprint, ReadMiss};
use std::sync::Mutex;

use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

pub(crate) enum PartitionProbe<'view> {
    Absent,
    Cold,
    Resident(&'view PartitionState),
}

struct FullReadRound<'source, 'round> {
    source: &'source dyn PartitionAccess,
    source_index: usize,
    misses: &'round Mutex<MissFootprint>,
}

impl<'source> FullReadRound<'source, '_> {
    fn read_partition(
        &self,
        source: &'source dyn PartitionAccess,
        id: PartitionId,
        base: bool,
    ) -> Option<&'source PartitionState> {
        match source.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            PartitionProbe::Absent => None,
            PartitionProbe::Cold => {
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

impl PartitionAccess for FullReadRound<'_, '_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.read_partition(self.source, id, false)
    }

    fn base_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.read_partition(self.source.base_source()?, id, true)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for FullReadRound<'_, '_> {
    fn base_current_source(&self) -> Option<&dyn crate::storage::overlay::CurrentPartitionAccess> {
        self.source
            .base_source()
            .map(|source| source as &dyn crate::storage::overlay::CurrentPartitionAccess)
    }
    fn base_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        PartitionAccess::base_partition(self, id)
            .map(crate::storage::residency::RelationalRegionHeadView::from_owned_partition)
    }
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        crate::storage::overlay::PartitionAccess::get_partition(self, id)
            .map(crate::storage::residency::RelationalRegionHeadView::from_owned_partition)
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::{PartitionHeadProbe as Head, PartitionProbe as Full};
        match crate::storage::overlay::PartitionAccess::probe_full_partition(self, id) {
            Full::Absent => Head::Absent,
            Full::Cold => Head::Cold,
            Full::Resident(partition) => Head::Resident(
                crate::storage::residency::RelationalRegionHeadView::from_owned_partition(
                    partition,
                ),
            ),
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::storage::overlay::PartitionAccess::demand_full_partition(self, execution, id)
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
pub(crate) fn probe_full<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &dyn PartitionAccess,
    mut read: impl FnMut(&dyn PartitionAccess) -> T,
) -> Result<T, super::RelationalResidencyDemandFailure> {
    probe_full_sources(execution, [source], |[view]| read(view))
}

/// Related views share one miss ledger, including overlay before-images.
/// The fixed source array and inline miss footprint need no heap allocation.
pub(crate) fn probe_full_sources<const N: usize, T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    sources: [&dyn PartitionAccess; N],
    mut read: impl FnMut([&dyn PartitionAccess; N]) -> T,
) -> Result<T, super::RelationalResidencyDemandFailure> {
    discover(
        execution,
        |misses| {
            let views: [_; N] = std::array::from_fn(|source_index| FullReadRound {
                source: sources[source_index],
                source_index,
                misses,
            });
            read(std::array::from_fn(|index| {
                &views[index] as &dyn PartitionAccess
            }))
        },
        |miss| {
            let source = sources[miss.source_index];
            let source = if miss.base {
                source
                    .base_source()
                    .expect("a recorded base miss has a base source")
            } else {
                source
            };
            source.demand_full_partition(execution, miss.partition)
        },
        || {
            for source in sources {
                for id in source.partition_ids_iter() {
                    source.demand_full_partition(execution, id)?;
                }
                if let Some(base) = source.base_source() {
                    for id in base.partition_ids_iter() {
                        base.demand_full_partition(execution, id)?;
                    }
                }
            }
            Ok(())
        },
    )
}
