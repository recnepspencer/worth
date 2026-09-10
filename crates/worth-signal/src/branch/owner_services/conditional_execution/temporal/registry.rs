use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, Weak};

use crate::branch::owner_services::{SignalOwnerLifecycleIdentity, SignalOwnerOperationAdmission};
use crate::data::retained_storage::{
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};

use super::partition::{SignalConditionalTemporalCell, SignalConditionalTemporalState};
use super::retention::{map_retention_denial, partition_charge};
use super::SignalConditionalTemporalPartitionDenial as Denial;

const MAXIMUM_CLOSE_BATCH: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SignalConditionalTemporalPartitionId(u64);

#[cfg(test)]
impl SignalConditionalTemporalPartitionId {
    pub(super) fn ordinal(self) -> u64 {
        self.0
    }
}

/// The owner retains registered cells; callers only retain weak capabilities.
pub(in crate::branch::owner_services) struct SignalConditionalTemporalRegistry {
    owner_runtime_instance_id: u64,
    owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    state: Mutex<SignalConditionalTemporalRegistryState>,
}

#[derive(Default)]
struct SignalConditionalTemporalRegistryState {
    next_partition_id: u64,
    cells: BTreeMap<SignalConditionalTemporalPartitionId, Arc<SignalConditionalTemporalCell>>,
}

pub(in crate::branch::owner_services) struct SignalConditionalTemporalCloseBatch {
    cells: [Option<Arc<SignalConditionalTemporalCell>>; MAXIMUM_CLOSE_BATCH],
}

impl SignalConditionalTemporalCloseBatch {
    pub(in crate::branch::owner_services) fn is_empty(&self) -> bool {
        self.cells[0].is_none()
    }
}

impl SignalConditionalTemporalRegistry {
    pub(in crate::branch::owner_services) fn new(
        owner_runtime_instance_id: u64,
        owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    ) -> Self {
        Self {
            owner_runtime_instance_id,
            owner_lifecycle_identity,
            state: Mutex::new(SignalConditionalTemporalRegistryState::default()),
        }
    }

    pub(super) fn register(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        maximum_active_wakes: NonZeroUsize,
        ledger: &Arc<SignalConditionalRetentionLedger>,
    ) -> Result<
        (
            SignalConditionalTemporalPartitionId,
            Weak<SignalConditionalTemporalCell>,
            Arc<SignalConditionalRetentionReservation>,
        ),
        Denial,
    > {
        admission
            .authorize(
                self.owner_runtime_instance_id,
                self.owner_lifecycle_identity,
            )
            .map_err(|_| Denial::PartitionAdmissionMismatch)?;
        let charge = partition_charge(maximum_active_wakes.get())
            .map_err(|_| Denial::StorageChargeOverflow)?;
        let reservation = ledger
            .reserve_temporal(1, maximum_active_wakes.get(), charge)
            .map_err(map_retention_denial)?;
        let mut state = self.state.lock().map_err(|_| Denial::RegistryQuarantined)?;
        let next = state
            .next_partition_id
            .checked_add(1)
            .ok_or(Denial::IdentityExhausted)?;
        let id = SignalConditionalTemporalPartitionId(state.next_partition_id);
        let custody = Arc::new(reservation);
        let cell = Arc::new(SignalConditionalTemporalCell {
            owner_runtime_instance_id: self.owner_runtime_instance_id,
            owner_lifecycle_identity: self.owner_lifecycle_identity,
            state: Mutex::new(SignalConditionalTemporalState {
                temporal: Default::default(),
                maximum_active_wakes: maximum_active_wakes.get(),
                quarantined: false,
                #[cfg(test)]
                promotion_fault: None,
                custody: Arc::clone(&custody),
            }),
        });
        let weak = Arc::downgrade(&cell);
        state.cells.insert(id, cell);
        state.next_partition_id = next;
        Ok((id, weak, custody))
    }

    pub(super) fn retire(&self, id: SignalConditionalTemporalPartitionId) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let cell = state.cells.remove(&id);
        if state.cells.is_empty() {
            state.cells = BTreeMap::new();
        }
        drop(state);
        // Destruction may scale with the admitted wake bound, outside membership locking.
        drop(cell);
    }

    pub(in crate::branch::owner_services) fn take_close_batch(
        &self,
        maximum: usize,
    ) -> SignalConditionalTemporalCloseBatch {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut cells = std::array::from_fn(|_| None);
        for slot in cells.iter_mut().take(maximum.min(MAXIMUM_CLOSE_BATCH)) {
            let Some((_, cell)) = state.cells.pop_first() else {
                break;
            };
            *slot = Some(cell);
        }
        if state.cells.is_empty() {
            state.cells = BTreeMap::new();
        }
        SignalConditionalTemporalCloseBatch { cells }
    }

    #[cfg(test)]
    pub(super) fn live_count(&self) -> usize {
        self.state.lock().unwrap().cells.len()
    }
}
