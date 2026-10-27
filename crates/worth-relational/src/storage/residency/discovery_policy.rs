//! One discard, load and bounded escalation policy for every residency level.
use super::RelationalResidencyDemandFailure;
use crate::identity::data::PartitionId;
use std::sync::Mutex;

const MAX_MISSES_PER_ROUND: usize = 16;
const MAX_DISCOVERY_ROUNDS: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct ReadMiss {
    pub(super) source_index: usize,
    pub(super) base: bool,
    pub(super) partition: PartitionId,
}

impl ReadMiss {
    fn key(self) -> (usize, bool, PartitionId) {
        (self.source_index, self.base, self.partition)
    }
}

pub(super) enum MissFootprint {
    Regions([Option<ReadMiss>; MAX_MISSES_PER_ROUND]),
    WholeSource(ReadMiss),
}

impl Default for MissFootprint {
    fn default() -> Self {
        Self::Regions([None; MAX_MISSES_PER_ROUND])
    }
}

impl MissFootprint {
    pub(super) fn record(&mut self, miss: ReadMiss) {
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
    pub(super) fn first(&self) -> Option<ReadMiss> {
        match self {
            Self::Regions(ids) => ids[0],
            Self::WholeSource(id) => Some(*id),
        }
    }
}

pub(super) fn discover<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(&Mutex<MissFootprint>) -> T,
    mut load_miss: impl FnMut(ReadMiss) -> Result<(), RelationalResidencyDemandFailure>,
    mut load_all: impl FnMut() -> Result<(), RelationalResidencyDemandFailure>,
) -> Result<T, RelationalResidencyDemandFailure> {
    for round_index in 0..=MAX_DISCOVERY_ROUNDS + 1 {
        crate::execution::check_request(execution)
            .map_err(RelationalResidencyDemandFailure::Request)?;
        let misses = Mutex::new(MissFootprint::default());
        let answer = read(&misses);
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
            load_all()?;
        } else if let MissFootprint::Regions(mut ids) = misses {
            ids.sort_unstable_by_key(|miss| miss.map(ReadMiss::key));
            for miss in ids.into_iter().flatten() {
                load_miss(miss)?;
            }
        }
    }
    unreachable!("the final round either accepts or refuses an unresolved miss")
}
