use std::sync::{Arc, Mutex};

use crate::history::data::RecordAllocationClass;
use crate::identity::data::PartitionId;

use super::RecordIdentityState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReservationOrigin {
    AppendFrontier,
    Reclaimed,
}

#[derive(Debug)]
pub(super) struct RecordSlotReservation {
    authority: Arc<Mutex<RecordIdentityState>>,
    class: RecordAllocationClass,
    partition_id: PartitionId,
    pub(super) slot: usize,
    generation: u32,
    pub(super) origin: ReservationOrigin,
    consumed: bool,
}

impl RecordSlotReservation {
    pub(super) fn coordinates(&self) -> (RecordAllocationClass, PartitionId, usize, u32) {
        (self.class, self.partition_id, self.slot, self.generation)
    }

    pub(super) fn new(
        authority: Arc<Mutex<RecordIdentityState>>,
        class: RecordAllocationClass,
        partition_id: PartitionId,
        slot: usize,
        generation: u32,
        origin: ReservationOrigin,
    ) -> Self {
        Self {
            authority,
            class,
            partition_id,
            slot,
            generation,
            origin,
            consumed: false,
        }
    }

    pub(super) fn consume(&mut self) {
        self.remove_pending();
        self.consumed = true;
    }

    fn remove_pending(&self) {
        let mut state = self
            .authority
            .lock()
            .expect("record identity lock poisoned");
        let key = (self.class, self.partition_id, self.slot);
        let exact = state
            .pending
            .get(&key)
            .is_some_and(|reservation| reservation.generation == self.generation);
        if exact {
            state.pending.remove(&key);
        }
    }
}

impl Drop for RecordSlotReservation {
    fn drop(&mut self) {
        if self.consumed {
            return;
        }
        let mut state = self
            .authority
            .lock()
            .expect("record identity lock poisoned");
        let key = (self.class, self.partition_id, self.slot);
        let exact = state
            .pending
            .get(&key)
            .is_some_and(|reservation| reservation.generation == self.generation);
        if exact {
            state.pending.remove(&key);
            if self.origin == ReservationOrigin::Reclaimed {
                state.reusable.insert(key);
            }
        }
    }
}
