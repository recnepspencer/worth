pub mod allocation_batch;
pub mod checkpoint_snapshot;
pub mod observation;
pub mod partition_frontier;
pub use allocation_batch::MutationRecordAllocations;
pub use observation::{
    RecordReservationObservation, RecordReservationObservationDenial, RecordReservationReport,
};
pub mod pending_allocations;
pub mod reclaimed;
pub mod reclamation;
pub mod record_ref;
pub mod recovery;
pub mod reservation;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::history::data::canonical_record_allocation::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::storage::substrate::SharedMap;
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;

pub use pending_allocations::PendingRecordAllocations;
pub use reclaimed::ReclaimedRecordSlot;
use reservation::{RecordSlotReservation, ReservationOrigin};

type RecordSlotKey = (RecordAllocationClass, PartitionId, usize);
type RecordFrontierKey = (RecordAllocationClass, PartitionId);

#[derive(Debug, Clone, Default)]
struct RecordIdentityState {
    reusable: SharedMap<RecordSlotKey, ()>,
    next_slots: SharedMap<RecordFrontierKey, usize>,
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
pub struct RecordIdentitySubsystem {
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
    pub fn begin_allocations(&self) -> PendingRecordAllocations {
        let staged_replay_allocations = self.lock().staged_replay_allocations.take();
        PendingRecordAllocations::new(self.clone(), staged_replay_allocations)
    }

    pub fn admit_reclaimed(&self, reclaimed: ReclaimedRecordSlot) {
        self.lock().admit_reclaimed(reclaimed);
    }

    fn reserve(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<(RecordSlotReservation, u32, RecordAllocationOrigin), RecordAllocationDenial> {
        let mut state = self.lock();
        let reusable = state
            .reusable
            .lower_bound(&(class, partition_id, 0))
            .map(|(key, ())| *key)
            .filter(|key| key.0 == class && key.1 == partition_id);
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
        observation::reserved(&self.state, class, partition_id, slot, generation);
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
        self.state.lock().expect(/* Proof: The identity mutex protects only native checked reservations and container updates, with no user code or fallible I/O under its guard; overflow returns before mutation. */ "record identity lock poisoned")
    }
}

impl RecordIdentitySubsystem {
    pub fn detached(&self) -> Self {
        let mut state = self.lock().clone();
        state.staged_replay_allocations = None;
        let fork = Self {
            state: Arc::new(Mutex::new(state)),
        };
        fork.release_unowned_reservations();
        fork
    }
}
