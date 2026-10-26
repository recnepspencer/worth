use std::collections::BTreeMap;
use std::sync::Arc;

use crate::history::data::{
    CanonicalRecordAllocation, RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::transactions::data::RecordAllocationDenial;

use super::record_ref::{record_partition, record_slot};
use super::reservation::{RecordSlotReservation, ReservationOrigin};
use super::{
    PendingRecordReservation, ReclaimedRecordSlot, RecordIdentitySubsystem, RecordSlotKey,
};

impl RecordIdentitySubsystem {
    /// Release copied/restored pending entries without a live owning handle.
    /// A detached fork calls this on its own copied state; recovery calls it
    /// after replay consumed its canonical reservations. Ordinary origin-sensitive
    /// RAII returns reclaimed slots while append-frontier gaps stay burned.
    pub(crate) fn release_unowned_reservations(&self) {
        let reservations = {
            let state = self.lock();
            state
                .pending
                .iter()
                .map(|(&(class, partition_id, slot), pending)| {
                    let origin = match pending.origin {
                        RecordAllocationOrigin::AppendFrontier => ReservationOrigin::AppendFrontier,
                        RecordAllocationOrigin::Reclaimed { .. } => ReservationOrigin::Reclaimed,
                    };
                    RecordSlotReservation::new(
                        Arc::clone(&self.state),
                        class,
                        partition_id,
                        slot,
                        pending.generation,
                        origin,
                    )
                })
                .collect::<Vec<_>>()
        };
        drop(reservations);
    }

    pub(crate) fn stage_replay_allocations_with_leading_gaps(
        &self,
        allocations: Vec<CanonicalRecordAllocation>,
    ) -> Result<(), &'static str> {
        if self.lock().staged_replay_allocations.is_some() {
            return Err("record allocation replay evidence is already staged");
        }
        let mut first_append_slots = BTreeMap::new();
        for allocation in &allocations {
            if allocation.origin() != RecordAllocationOrigin::AppendFrontier {
                continue;
            }
            let key = (allocation.class(), record_partition(allocation.record()));
            let slot = record_slot(allocation.record());
            first_append_slots
                .entry(key)
                .and_modify(|first: &mut usize| *first = (*first).min(slot))
                .or_insert(slot);
        }
        let mut state = self.lock();
        for (key, first_slot) in first_append_slots {
            let frontier = state.next_slots.entry(key).or_default();
            if *frontier < first_slot {
                *frontier = first_slot;
            }
        }
        state.staged_replay_allocations = Some(allocations);
        Ok(())
    }

    pub(crate) fn clear_staged_replay_allocations(&self) -> bool {
        self.lock().staged_replay_allocations.take().is_some()
    }

    pub(crate) fn reusable_snapshot(&self) -> Vec<RecordSlotKey> {
        self.lock().reusable.iter().copied().collect()
    }

    pub(crate) fn frontier_snapshot(&self) -> Vec<(RecordAllocationClass, PartitionId, usize)> {
        self.lock()
            .next_slots
            .iter()
            .map(|(&(class, partition_id), &next_slot)| (class, partition_id, next_slot))
            .collect()
    }

    pub(crate) fn generation_snapshot(
        &self,
    ) -> Vec<(RecordAllocationClass, PartitionId, u64, u32)> {
        self.lock()
            .generation_high_water
            .iter()
            .map(|(&(class, partition_id, slot), &generation)| {
                (class, partition_id, slot as u64, generation)
            })
            .collect()
    }

    pub(crate) fn pending_snapshot(
        &self,
    ) -> Vec<(
        RecordAllocationClass,
        PartitionId,
        u64,
        u32,
        RecordAllocationOrigin,
    )> {
        self.lock()
            .pending
            .iter()
            .map(|(&(class, partition_id, slot), reservation)| {
                (
                    class,
                    partition_id,
                    slot as u64,
                    reservation.generation,
                    reservation.origin,
                )
            })
            .collect()
    }

    pub(crate) fn restore_reusable(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
    ) {
        self.admit_reclaimed(ReclaimedRecordSlot::new(class, partition_id, slot));
    }

    pub(crate) fn restore_frontier(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        next_slot: usize,
    ) {
        let mut state = self.lock();
        let frontier = state.next_slots.entry((class, partition_id)).or_default();
        *frontier = (*frontier).max(next_slot);
    }

    pub(crate) fn restore_generation(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        generation: u32,
    ) {
        let mut state = self.lock();
        let high_water = state
            .generation_high_water
            .entry((class, partition_id, slot))
            .or_default();
        *high_water = (*high_water).max(generation);
    }

    pub(crate) fn restore_pending(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        generation: u32,
        origin: RecordAllocationOrigin,
    ) -> Result<(), &'static str> {
        let mut state = self.lock();
        let key = (class, partition_id, slot);
        if state.pending.contains_key(&key) {
            return Err("duplicate durable record reservation");
        }
        if state.generation_high_water.get(&key).copied() != Some(generation) {
            return Err("durable record reservation generation is not at high water");
        }
        match origin {
            RecordAllocationOrigin::AppendFrontier => {
                let frontier = state
                    .next_slots
                    .get(&(class, partition_id))
                    .copied()
                    .unwrap_or(0);
                if frontier <= slot {
                    return Err("durable append reservation is beyond its frontier");
                }
            }
            RecordAllocationOrigin::Reclaimed { .. } => {
                if state.reusable.contains(&key) {
                    return Err("durable reclaimed reservation is also reusable");
                }
            }
        }
        state
            .pending
            .insert(key, PendingRecordReservation { generation, origin });
        Ok(())
    }

    pub(super) fn reserve_exact(
        &self,
        ordinal: u64,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        generation: u32,
        origin: RecordAllocationOrigin,
    ) -> Result<RecordSlotReservation, RecordAllocationDenial> {
        let mut state = self.lock();
        let key = (class, partition_id, slot);
        let restored_origin = match origin {
            RecordAllocationOrigin::AppendFrontier => ReservationOrigin::AppendFrontier,
            RecordAllocationOrigin::Reclaimed { .. } => ReservationOrigin::Reclaimed,
        };
        if state.pending.get(&key).copied() == Some(PendingRecordReservation { generation, origin })
        {
            return Ok(RecordSlotReservation::new(
                Arc::clone(&self.state),
                class,
                partition_id,
                slot,
                generation,
                restored_origin,
            ));
        }
        let prior_generation = match origin {
            RecordAllocationOrigin::AppendFrontier => 0,
            RecordAllocationOrigin::Reclaimed { prior_generation } => prior_generation,
        };
        let current_generation = state
            .generation_high_water
            .get(&(class, partition_id, slot))
            .copied()
            .unwrap_or(0);
        let expected_generation =
            prior_generation
                .checked_add(1)
                .ok_or(RecordAllocationDenial::GenerationExhausted {
                    class,
                    partition_id,
                    slot,
                })?;
        if current_generation != prior_generation || generation != expected_generation {
            return Err(RecordAllocationDenial::ReplayGenerationMismatch {
                ordinal,
                class,
                partition_id,
                slot,
                expected_generation,
                observed_generation: generation,
            });
        }
        let reservation_origin = match origin {
            RecordAllocationOrigin::Reclaimed { .. } => {
                if !state.reusable.remove(&(class, partition_id, slot)) {
                    return Err(RecordAllocationDenial::ReplaySlotUnavailable {
                        ordinal,
                        class,
                        partition_id,
                        slot,
                    });
                }
                ReservationOrigin::Reclaimed
            }
            RecordAllocationOrigin::AppendFrontier => {
                let expected_slot = state
                    .next_slots
                    .get(&(class, partition_id))
                    .copied()
                    .unwrap_or(0);
                if slot != expected_slot {
                    return Err(RecordAllocationDenial::ReplayAppendFrontierMismatch {
                        ordinal,
                        class,
                        partition_id,
                        expected_slot,
                        observed_slot: slot,
                    });
                }
                let next =
                    slot.checked_add(1)
                        .ok_or(RecordAllocationDenial::SlotFrontierExhausted {
                            class,
                            partition_id,
                        })?;
                state.next_slots.insert((class, partition_id), next);
                ReservationOrigin::AppendFrontier
            }
        };
        state
            .generation_high_water
            .insert((class, partition_id, slot), generation);
        state.pending.insert(
            (class, partition_id, slot),
            PendingRecordReservation { generation, origin },
        );
        Ok(RecordSlotReservation::new(
            Arc::clone(&self.state),
            class,
            partition_id,
            slot,
            generation,
            reservation_origin,
        ))
    }
}
