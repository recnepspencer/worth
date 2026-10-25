//! Full-history rounds over an explicit source. Only a miss-free round escapes.
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
    misses: &'round Mutex<MissFootprint<'source>>,
}

const MAX_MISSES_PER_ROUND: usize = 16;

/// Inline discovery needs no unaccounted allocation. A wider footprint uses
/// the same declared whole-source escalation as the retry ceiling.
#[derive(Clone, Copy)]
struct ReadMiss<'source> {
    source: &'source dyn PartitionAccess,
    source_index: usize,
    base: bool,
    partition: PartitionId,
}

impl ReadMiss<'_> {
    fn key(&self) -> (usize, bool, PartitionId) {
        (self.source_index, self.base, self.partition)
    }
}

enum MissFootprint<'source> {
    Regions([Option<ReadMiss<'source>>; MAX_MISSES_PER_ROUND]),
    WholeSource(ReadMiss<'source>),
}

impl Default for MissFootprint<'_> {
    fn default() -> Self {
        Self::Regions([None; MAX_MISSES_PER_ROUND])
    }
}

impl<'source> MissFootprint<'source> {
    fn record(&mut self, miss: ReadMiss<'source>) {
        if let Self::Regions(ids) = self {
            if ids.iter().flatten().any(|id| id.key() == miss.key()) {
                return;
            }
            if let Some(slot) = ids.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(miss);
            } else {
                *self = Self::WholeSource(miss);
            }
        }
    }

    fn first(&self) -> Option<ReadMiss<'source>> {
        match self {
            Self::Regions(ids) => ids[0],
            Self::WholeSource(id) => Some(*id),
        }
    }
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
                        source,
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
    fn has_read_misses(&self) -> bool {
        self.misses
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .first()
            .is_some()
    }
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.read_partition(self.source, id, false)
    }

    fn base_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.read_partition(self.source.base_source()?, id, true)
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
    const MAX_DISCOVERY_ROUNDS: usize = 16;
    for round_index in 0..=MAX_DISCOVERY_ROUNDS + 1 {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        let misses = Mutex::new(MissFootprint::default());
        let views: [_; N] = std::array::from_fn(|source_index| FullReadRound {
            source: sources[source_index],
            source_index,
            misses: &misses,
        });
        let answer = read(std::array::from_fn(|index| {
            &views[index] as &dyn PartitionAccess
        }));
        let misses = misses
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        let Some(first_miss) = misses.first() else {
            return Ok(answer);
        };
        drop(answer);
        if round_index > MAX_DISCOVERY_ROUNDS {
            return Err(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(
                    first_miss.partition,
                ),
                cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
            }
            .into());
        }
        if round_index == MAX_DISCOVERY_ROUNDS || matches!(misses, MissFootprint::WholeSource(_)) {
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
        } else if let MissFootprint::Regions(mut ids) = misses {
            ids.sort_unstable_by_key(|miss| miss.map(|miss| miss.key()));
            for miss in ids.into_iter().flatten() {
                miss.source
                    .demand_full_partition(execution, miss.partition)?;
            }
        }
    }
    unreachable!("the final round either accepts or refuses an unresolved miss")
}
