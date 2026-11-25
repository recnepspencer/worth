//! Discard provisional answers and load only observed misses until progress completes.
use super::RelationalResidencyDemandFailure;
use crate::identity::data::PartitionId;
use std::sync::Mutex;

const MAX_MISSES_PER_ROUND: usize = 16;

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

pub(super) struct MissFootprint([Option<ReadMiss>; MAX_MISSES_PER_ROUND]);
impl Default for MissFootprint {
    fn default() -> Self {
        Self([None; MAX_MISSES_PER_ROUND])
    }
}
impl MissFootprint {
    pub(super) fn record(&mut self, miss: ReadMiss) {
        if self.0.iter().flatten().any(|id| id.key() == miss.key()) {
            return;
        }
        if let Some(slot) = self.0.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(miss);
        }
        // Overflow remains a miss in this provisional round. The recorded
        // batch resolves first; later rounds discover remaining cold regions.
    }
    pub(super) fn first(&self) -> Option<ReadMiss> {
        self.0[0]
    }
}

pub(super) fn discover<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(&Mutex<MissFootprint>) -> T,
    mut load_miss: impl FnMut(ReadMiss) -> Result<(), RelationalResidencyDemandFailure>,
) -> Result<T, RelationalResidencyDemandFailure> {
    let mut previous_first = None;
    loop {
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
        if previous_first == Some(first_miss.key()) {
            // A successful demand must resolve its exact immutable source miss.
            // No progress is unavailable state, not proof of corrupt bytes.
            return Err(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(
                    first_miss.partition,
                ),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            }
            .into());
        }
        previous_first = Some(first_miss.key());
        let mut ids = misses.0;
        ids.sort_unstable_by_key(|miss| miss.map(ReadMiss::key));
        for miss in ids.into_iter().flatten() {
            load_miss(miss)?;
        }
    }
}
