pub mod allocation_batch;
pub mod allocation_source;
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
mod replay_gaps;
pub mod reservation;

#[cfg(test)]
mod replay_block_tests;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::history::data::canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation, RecordAllocationClass,
    RecordAllocationOrigin,
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
    staged_replay_allocations: Option<StagedReplayAllocations>,
    /// The live append slots the staged commit's evidence admitted, per class
    /// and partition; each live replay allocation consumes one of them.
    replay_slots: BTreeMap<RecordFrontierKey, std::collections::BTreeSet<usize>>,
    /// Append slots tail replay skipped, which a later commit may publish (#389).
    replay_gaps: replay_gaps::ReplayGaps,
}

/// One durable commit's allocation evidence, staged for its replay.
#[derive(Debug, Clone, Default)]
pub struct StagedReplayAllocations {
    live: Vec<CanonicalRecordAllocation>,
    cancelled: Vec<CancelledRecordAllocation>,
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
        match self.lock().staged_replay_allocations.take() {
            Some(staged) => PendingRecordAllocations::replaying(self.clone(), staged),
            None => PendingRecordAllocations::new(self.clone(), None),
        }
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
        state.replay_slots.clear();
        state.replay_gaps.clear();
        let fork = Self {
            state: Arc::new(Mutex::new(state)),
        };
        fork.release_unowned_reservations();
        fork
    }

    /// Detach a snapshot funded by `request` before cloning any owned maps.
    /// Reclaimed pending slots are returned directly to `reusable` without
    /// cloning `pending` or `staged_replay_allocations`.
    pub fn detached_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (Self, Option<worth_execution::ExecutionMemoryReservation>),
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
        crate::execution::request_custody::check_request(request)?;
        let state = self.lock();
        let bytes = arc_layout_size::<Mutex<RecordIdentityState>>()
            .and_then(|n| {
                n.checked_add(btree_map_bytes::<RecordSlotKey, u32>(
                    state.generation_high_water.len(),
                )?)
            })
            .ok_or(Cause::MemoryOverflow)?;
        let reservation = request
            .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Cause::MemoryOverflow)?)
            .map_err(Cause::from)?;
        let mut reusable = state.reusable.clone();
        for (&key, pending) in &state.pending {
            if matches!(pending.origin, RecordAllocationOrigin::Reclaimed { .. }) {
                reusable.insert_for_request(key, (), request)?;
            }
        }
        let next_slots = state.next_slots.clone();
        let generation_high_water = state.generation_high_water.clone();
        drop(state);
        Ok((
            Self {
                state: Arc::new(Mutex::new(RecordIdentityState {
                    reusable,
                    next_slots,
                    generation_high_water,
                    pending: BTreeMap::new(),
                    staged_replay_allocations: None,
                    replay_slots: BTreeMap::new(),
                    replay_gaps: Default::default(),
                })),
            },
            reservation,
        ))
    }
}

fn arc_layout_size<T>() -> Option<usize> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .map(|(layout, _)| layout.pad_to_align().size())
}

fn btree_map_bytes<K, V>(len: usize) -> Option<usize> {
    if len == 0 {
        return Some(0);
    }
    let entry = std::mem::size_of::<K>().checked_add(std::mem::size_of::<V>())?;
    let node = entry
        .checked_mul(11)?
        .checked_add(std::mem::size_of::<usize>().checked_mul(12)?)?;
    // Conservative 2x node bound covers internal parent nodes and per-node
    // alignment/metadata padding across sequential BTreeMap clones.
    len.div_ceil(6).checked_mul(node)?.checked_mul(2)
}
