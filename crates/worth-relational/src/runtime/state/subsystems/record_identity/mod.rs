mod pending_allocations;
mod reclaimed;
mod record_ref;
mod recovery;
mod reservation;

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::history::data::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::RecordAllocationDenial;

pub(crate) use pending_allocations::PendingRecordAllocations;
pub(crate) use reclaimed::ReclaimedRecordSlot;
use reservation::{RecordSlotReservation, ReservationOrigin};

use super::RuntimeSubsystem;

type RecordSlotKey = (RecordAllocationClass, PartitionId, usize);
type RecordFrontierKey = (RecordAllocationClass, PartitionId);

#[derive(Debug, Clone, Default)]
struct RecordIdentityState {
    reusable: BTreeSet<RecordSlotKey>,
    next_slots: BTreeMap<RecordFrontierKey, usize>,
    generation_high_water: BTreeMap<RecordSlotKey, u32>,
    pending: BTreeMap<RecordSlotKey, PendingRecordReservation>,
    staged_replay_allocations: Option<Vec<CanonicalRecordAllocation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingRecordReservation {
    generation: u32,
    origin: RecordAllocationOrigin,
}

#[derive(Debug, Default)]
pub(crate) struct RecordIdentitySubsystem {
    state: Arc<Mutex<RecordIdentityState>>,
}

impl Clone for RecordIdentitySubsystem {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl RecordIdentitySubsystem {
    pub(crate) fn begin_allocations(&self) -> PendingRecordAllocations {
        let staged_replay_allocations = self.lock().staged_replay_allocations.take();
        PendingRecordAllocations::new(self.clone(), staged_replay_allocations)
    }

    pub(crate) fn admit_reclaimed(&self, reclaimed: ReclaimedRecordSlot) {
        let mut state = self.lock();
        state
            .reusable
            .insert((reclaimed.class, reclaimed.partition_id, reclaimed.slot));
        if let Some(next) = reclaimed.slot.checked_add(1) {
            let frontier = state
                .next_slots
                .entry((reclaimed.class, reclaimed.partition_id))
                .or_default();
            *frontier = (*frontier).max(next);
        }
    }

    fn reserve(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<(RecordSlotReservation, u32, RecordAllocationOrigin), RecordAllocationDenial> {
        let mut state = self.lock();
        let reusable = state
            .reusable
            .range((class, partition_id, 0)..=(class, partition_id, usize::MAX))
            .next()
            .copied();
        let (slot, origin) = reusable
            .map(|key| (key.2, ReservationOrigin::Reclaimed))
            .unwrap_or_else(|| {
                (
                    state
                        .next_slots
                        .get(&(class, partition_id))
                        .copied()
                        .unwrap_or(0),
                    ReservationOrigin::AppendFrontier,
                )
            });
        let prior_generation = state
            .generation_high_water
            .get(&(class, partition_id, slot))
            .copied()
            .unwrap_or(0);
        let generation =
            prior_generation
                .checked_add(1)
                .ok_or(RecordAllocationDenial::GenerationExhausted {
                    class,
                    partition_id,
                    slot,
                })?;
        match origin {
            ReservationOrigin::Reclaimed => {
                state.reusable.remove(&(class, partition_id, slot));
            }
            ReservationOrigin::AppendFrontier => {
                let next =
                    slot.checked_add(1)
                        .ok_or(RecordAllocationDenial::SlotFrontierExhausted {
                            class,
                            partition_id,
                        })?;
                state.next_slots.insert((class, partition_id), next);
            }
        }
        state
            .generation_high_water
            .insert((class, partition_id, slot), generation);
        let canonical_origin = match origin {
            ReservationOrigin::AppendFrontier => RecordAllocationOrigin::AppendFrontier,
            ReservationOrigin::Reclaimed => RecordAllocationOrigin::Reclaimed { prior_generation },
        };
        state.pending.insert(
            (class, partition_id, slot),
            PendingRecordReservation {
                generation,
                origin: canonical_origin,
            },
        );
        Ok((
            RecordSlotReservation::new(
                Arc::clone(&self.state),
                class,
                partition_id,
                slot,
                generation,
                origin,
            ),
            generation,
            canonical_origin,
        ))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RecordIdentityState> {
        self.state.lock().expect("record identity lock poisoned")
    }
}

impl RuntimeSubsystem for RecordIdentitySubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self::default()
    }

    fn fork(&self) -> Self {
        let mut state = self.lock().clone();
        state.staged_replay_allocations = None;
        let fork = Self {
            state: Arc::new(Mutex::new(state)),
        };
        fork.release_unowned_reservations();
        fork
    }
}
