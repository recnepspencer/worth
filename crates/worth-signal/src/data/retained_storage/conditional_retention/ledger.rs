use std::sync::{Arc, Mutex};

use crate::data::retained_storage::RetainedStorageCharge;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalConditionalTemporalBudget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalConditionalRetentionDenial {
    Closed,
    CapacityExhausted,
    Poisoned,
    InvalidTransfer,
}

/// Aggregate custody across all branch cells and evidence surviving those cells.
/// The mutex covers only checked count updates, never graph work or callbacks.
#[derive(Debug)]
pub(crate) struct SignalConditionalRetentionLedger {
    maximum_slots: usize,
    maximum_bytes: u64,
    maximum_partitions: usize,
    maximum_reserved_wakes: usize,
    state: Mutex<RetainedConditionalUsage>,
}

#[derive(Debug, Default)]
struct RetainedConditionalUsage {
    closed: bool,
    slots: usize,
    bytes: u64,
    partitions: usize,
    reserved_wakes: usize,
}

/// Descriptive owner usage. It carries no reservation or execution authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalConditionalRetentionObservation {
    retained_slots: usize,
    retained_bytes: u64,
}

impl SignalConditionalRetentionObservation {
    pub const fn retained_slots(self) -> usize {
        self.retained_slots
    }

    pub const fn retained_bytes(self) -> u64 {
        self.retained_bytes
    }
}

/// Move-only resource custody. This retains accounting, not the Signal owner.
#[derive(Debug)]
pub(crate) struct SignalConditionalRetentionReservation {
    ledger: Arc<SignalConditionalRetentionLedger>,
    slots: usize,
    bytes: u64,
    partitions: usize,
    reserved_wakes: usize,
}

impl SignalConditionalRetentionLedger {
    pub(crate) fn new(
        budget: SignalConditionalEvaluationBudget,
        temporal: SignalConditionalTemporalBudget,
    ) -> Arc<Self> {
        Arc::new(Self {
            maximum_slots: budget.maximum_retained_slots,
            maximum_bytes: budget.maximum_retained_bytes,
            maximum_partitions: temporal.maximum_live_partitions,
            maximum_reserved_wakes: temporal.maximum_reserved_active_wakes,
            state: Mutex::new(RetainedConditionalUsage::default()),
        })
    }

    pub(crate) fn reserve(
        self: &Arc<Self>,
        slots: usize,
        charge: RetainedStorageCharge,
    ) -> Result<SignalConditionalRetentionReservation, SignalConditionalRetentionDenial> {
        self.reserve_axes(slots, 0, 0, charge)
    }

    /// Temporal reservations never consume evaluation slots. Their independent
    /// quotas and representation bytes remain reserved until custody is dropped.
    pub(crate) fn reserve_temporal(
        self: &Arc<Self>,
        partitions: usize,
        reserved_wakes: usize,
        charge: RetainedStorageCharge,
    ) -> Result<SignalConditionalRetentionReservation, SignalConditionalRetentionDenial> {
        self.reserve_axes(0, partitions, reserved_wakes, charge)
    }

    fn reserve_axes(
        self: &Arc<Self>,
        slots: usize,
        partitions: usize,
        reserved_wakes: usize,
        charge: RetainedStorageCharge,
    ) -> Result<SignalConditionalRetentionReservation, SignalConditionalRetentionDenial> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| SignalConditionalRetentionDenial::Poisoned)?;
        if state.closed {
            return Err(SignalConditionalRetentionDenial::Closed);
        }
        let bytes = charge
            .bytes()
            .checked_add(SignalConditionalRetentionReservation::handle_bytes())
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        let next_slots = state
            .slots
            .checked_add(slots)
            .filter(|total| *total <= self.maximum_slots)
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        let next_bytes = state
            .bytes
            .checked_add(bytes)
            .filter(|total| *total <= self.maximum_bytes)
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        let next_partitions = state
            .partitions
            .checked_add(partitions)
            .filter(|total| *total <= self.maximum_partitions)
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        let next_wakes = state
            .reserved_wakes
            .checked_add(reserved_wakes)
            .filter(|total| *total <= self.maximum_reserved_wakes)
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        state.slots = next_slots;
        state.bytes = next_bytes;
        state.partitions = next_partitions;
        state.reserved_wakes = next_wakes;
        Ok(SignalConditionalRetentionReservation {
            ledger: Arc::clone(self),
            slots,
            bytes,
            partitions,
            reserved_wakes,
        })
    }

    pub(crate) fn close(&self) {
        // Closing must still fence a poisoned ledger. No user callback or
        // fallible payload destruction occurs while this small lock is held.
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .closed = true;
    }

    pub(crate) fn observe(&self) -> SignalConditionalRetentionObservation {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        SignalConditionalRetentionObservation {
            retained_slots: state.slots,
            retained_bytes: state.bytes,
        }
    }

    #[cfg(test)]
    pub(crate) fn usage(&self) -> (usize, u64) {
        let observation = self.observe();
        (observation.retained_slots(), observation.retained_bytes())
    }

    #[cfg(test)]
    pub(crate) fn temporal_usage(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.partitions, state.reserved_wakes)
    }
}

impl SignalConditionalRetentionReservation {
    /// Descendants reserve against the same owner; this reference grants no
    /// capacity and remains subject to the owner's close fence.
    pub(crate) fn ledger(&self) -> Arc<SignalConditionalRetentionLedger> {
        Arc::clone(&self.ledger)
    }

    /// The new backing allocation already includes this embedded handle.
    /// Split must not charge that same inline handle a second time.
    pub(crate) fn split_embedded(
        &mut self,
        allocation: RetainedStorageCharge,
    ) -> Result<Self, SignalConditionalRetentionDenial> {
        let payload = allocation
            .checked_sub(
                RetainedStorageCharge::capacity::<Self>(1)
                    .map_err(|_| SignalConditionalRetentionDenial::InvalidTransfer)?,
            )
            .map_err(|_| SignalConditionalRetentionDenial::InvalidTransfer)?;
        self.split(0, payload)
    }

    fn handle_bytes() -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    /// Additional payload capacity must be admitted before the owner grows it.
    /// The single reservation grows in place, without a per-write lease list.
    pub(crate) fn grow(
        &mut self,
        charge: RetainedStorageCharge,
    ) -> Result<(), SignalConditionalRetentionDenial> {
        let mut state = self
            .ledger
            .state
            .lock()
            .map_err(|_| SignalConditionalRetentionDenial::Poisoned)?;
        if state.closed {
            return Err(SignalConditionalRetentionDenial::Closed);
        }
        let next_bytes = state
            .bytes
            .checked_add(charge.bytes())
            .filter(|total| *total <= self.ledger.maximum_bytes)
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        let reservation_bytes = self
            .bytes
            .checked_add(charge.bytes())
            .ok_or(SignalConditionalRetentionDenial::CapacityExhausted)?;
        state.bytes = next_bytes;
        self.bytes = reservation_bytes;
        Ok(())
    }

    /// Release conservative staging excess after the exact retained root charge
    /// is known. The reservation handle remains live and charged.
    pub(crate) fn shrink_payload_to(
        &mut self,
        payload: RetainedStorageCharge,
    ) -> Result<(), SignalConditionalRetentionDenial> {
        let target = payload
            .bytes()
            .checked_add(Self::handle_bytes())
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        let released = self
            .bytes
            .checked_sub(target)
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        let mut state = self
            .ledger
            .state
            .lock()
            .map_err(|_| SignalConditionalRetentionDenial::Poisoned)?;
        state.bytes = state
            .bytes
            .checked_sub(released)
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        self.bytes = target;
        Ok(())
    }

    /// Transfer already reserved payload and its new handle. Both resulting
    /// handles stay charged; no aggregate growth or owner liveness is required.
    /// The caller reserves room for future split handles before doing work.
    pub(crate) fn split(
        &mut self,
        slots: usize,
        charge: RetainedStorageCharge,
    ) -> Result<Self, SignalConditionalRetentionDenial> {
        let bytes = charge
            .bytes()
            .checked_add(Self::handle_bytes())
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        let remaining = self
            .bytes
            .checked_sub(bytes)
            .filter(|remaining| *remaining >= Self::handle_bytes())
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        let remaining_slots = self
            .slots
            .checked_sub(slots)
            .ok_or(SignalConditionalRetentionDenial::InvalidTransfer)?;
        let split = Self {
            ledger: Arc::clone(&self.ledger),
            slots,
            bytes,
            partitions: 0,
            reserved_wakes: 0,
        };
        self.bytes = remaining;
        self.slots = remaining_slots;
        Ok(split)
    }
}

impl Drop for SignalConditionalRetentionReservation {
    fn drop(&mut self) {
        let mut state = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        state.slots = state
            .slots
            .checked_sub(self.slots)
            .expect("reservation owns its slots");
        state.bytes = state
            .bytes
            .checked_sub(self.bytes)
            .expect("reservation owns its bytes");
        state.partitions = state
            .partitions
            .checked_sub(self.partitions)
            .expect("reservation owns its partitions");
        state.reserved_wakes = state
            .reserved_wakes
            .checked_sub(self.reserved_wakes)
            .expect("reservation owns its wake capacity");
    }
}
