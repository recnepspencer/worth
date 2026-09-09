use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge, RetainedStoragePreparationDenial,
};
use crate::data::telemetry::{InvalidationPerformedCounter, SignalInvalidationRealizedCounters};
use crate::logic::transaction::{SignalObservationCaptureGate, SignalObservationSurface};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct InvalidationPerformedCounterState {
    values: Arc<[AtomicU64; 24]>,
    capture_gate: SignalObservationCaptureGate,
}

impl Default for InvalidationPerformedCounterState {
    fn default() -> Self {
        Self {
            values: Arc::new(std::array::from_fn(|_| AtomicU64::new(0))),
            capture_gate: SignalObservationCaptureGate::default(),
        }
    }
}

impl InvalidationPerformedCounterState {
    pub(crate) fn initial_heap_charge(
    ) -> Result<RetainedStorageCharge, RetainedStoragePreparationDenial> {
        arc_allocation_charge::<[AtomicU64; 24]>()
    }

    pub(crate) fn with_capture_gate(capture_gate: SignalObservationCaptureGate) -> Self {
        Self {
            capture_gate,
            values: Arc::new(std::array::from_fn(|_| AtomicU64::new(0))),
        }
    }

    pub(crate) fn shared_values(&self) -> Arc<[AtomicU64; 24]> {
        Arc::clone(&self.values)
    }

    pub(crate) fn reset(&self) {
        for value in self.values.iter() {
            value.store(0, Ordering::Relaxed);
        }
    }

    pub(crate) fn begin_capture(&self) {
        self.reset();
    }

    pub(crate) fn add(&self, counter: InvalidationPerformedCounter, amount: u64) {
        if !self
            .capture_gate
            .captures(SignalObservationSurface::PerformedCounters)
        {
            return;
        }
        self.values[counter.index()].fetch_add(amount, Ordering::Relaxed);
    }

    pub(crate) fn set(&self, counter: InvalidationPerformedCounter, value: u64) {
        if !self
            .capture_gate
            .captures(SignalObservationSurface::PerformedCounters)
        {
            return;
        }
        self.values[counter.index()].store(value, Ordering::Relaxed);
    }

    pub(crate) fn record_max(&self, counter: InvalidationPerformedCounter, value: u64) {
        if !self
            .capture_gate
            .captures(SignalObservationSurface::PerformedCounters)
        {
            return;
        }
        self.values[counter.index()].fetch_max(value, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> SignalInvalidationRealizedCounters {
        SignalInvalidationRealizedCounters::from_values(std::array::from_fn(|index| {
            self.values[index].load(Ordering::Relaxed)
        }))
    }
}
