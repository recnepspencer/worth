//! Full-history rounds over an explicit source. Only a miss-free round escapes.
use std::sync::Mutex;

use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};

pub(crate) enum PartitionProbe<'view> {
    Absent,
    Cold,
    Resident(&'view PartitionState),
}

struct FullReadRound<'source> {
    source: &'source dyn PartitionAccess,
    misses: Mutex<MissFootprint>,
}

const MAX_MISSES_PER_ROUND: usize = 16;

/// Inline discovery needs no unaccounted allocation. A wider footprint uses
/// the same declared whole-source escalation as the retry ceiling.
enum MissFootprint {
    Regions([Option<PartitionId>; MAX_MISSES_PER_ROUND]),
    WholeSource(PartitionId),
}

impl Default for MissFootprint {
    fn default() -> Self {
        Self::Regions([None; MAX_MISSES_PER_ROUND])
    }
}

impl MissFootprint {
    fn record(&mut self, id: PartitionId) {
        if let Self::Regions(ids) = self {
            if ids.contains(&Some(id)) {
                return;
            }
            if let Some(slot) = ids.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(id);
            } else {
                *self = Self::WholeSource(id);
            }
        }
    }

    fn first(&self) -> Option<PartitionId> {
        match self {
            Self::Regions(ids) => ids[0],
            Self::WholeSource(id) => Some(*id),
        }
    }
}

impl PartitionAccess for FullReadRound<'_> {
    fn has_read_misses(&self) -> bool {
        self.misses
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .first()
            .is_some()
    }
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        match self.source.probe_full_partition(id) {
            PartitionProbe::Resident(partition) => Some(partition),
            PartitionProbe::Absent => None,
            PartitionProbe::Cold => {
                self.misses
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .record(id);
                None
            }
        }
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
    const MAX_DISCOVERY_ROUNDS: usize = 16;
    for round_index in 0..=MAX_DISCOVERY_ROUNDS + 1 {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        let round = FullReadRound {
            source,
            misses: Mutex::new(MissFootprint::default()),
        };
        let answer = read(&round);
        let misses = round
            .misses
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        let Some(first_miss) = misses.first() else {
            return Ok(answer);
        };
        drop(answer);
        if round_index > MAX_DISCOVERY_ROUNDS {
            return Err(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(first_miss),
                cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
            }
            .into());
        }
        if round_index == MAX_DISCOVERY_ROUNDS || matches!(misses, MissFootprint::WholeSource(_)) {
            for id in source.partition_ids_iter() {
                source.demand_full_partition(execution, id)?;
            }
        } else if let MissFootprint::Regions(mut ids) = misses {
            ids.sort_unstable();
            for id in ids.into_iter().flatten() {
                source.demand_full_partition(execution, id)?;
            }
        }
    }
    unreachable!("the final round either accepts or refuses an unresolved miss")
}
