use crate::data::error::SignalError;
use crate::data::proof::invalidation::progression::InvalidationWorkBindingAxes;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStoragePreparationDenial,
    SignalConditionalRetentionLedger as Ledger,
    SignalConditionalRetentionReservation as Reservation,
};
use crate::logic::evaluation::EvaluationWork;
use crate::logic::transaction::{SignalObservationCaptureGate, SignalObservationSurface};
use std::sync::{Arc, Mutex};
mod buffer;
mod graph;
mod preparation;
mod targets;
pub(crate) use buffer::PerformedWorkBuffer;
pub(crate) use preparation::PreparedPerformedWorkCapture;
pub(crate) use targets::PerformedTargetSnapshot;

#[derive(Debug)]
pub(crate) struct PerformedWorkCaptureState {
    capture_gate: SignalObservationCaptureGate,
    bindings: Arc<Mutex<PerformedWorkBuffer>>,
}
impl Default for PerformedWorkCaptureState {
    fn default() -> Self {
        Self::with_capture_gate(SignalObservationCaptureGate::default())
    }
}
impl PerformedWorkCaptureState {
    pub(crate) fn initial_heap_charge() -> Result<Charge, RetainedStoragePreparationDenial> {
        arc_allocation_charge::<Mutex<PerformedWorkBuffer>>()
    }
    pub(crate) fn with_capture_gate(capture_gate: SignalObservationCaptureGate) -> Self {
        Self {
            capture_gate,
            bindings: Arc::new(Mutex::new(PerformedWorkBuffer::default())),
        }
    }
    pub(crate) fn with_storage_custody(self, custody: Option<Arc<Reservation>>) -> Self {
        self.bindings
            .lock()
            .expect("fresh capture storage")
            .storage_custody = custody;
        self
    }
    pub(crate) fn ensure_available(&self) {
        drop(
            self.bindings
                .lock()
                .expect("performed work observation poisoned"),
        );
    }
    pub(crate) fn reset(&self) {
        self.bindings
            .lock()
            .expect("performed work observation poisoned")
            .clear();
    }
    pub(crate) fn shared_bindings(&self) -> Arc<Mutex<PerformedWorkBuffer>> {
        Arc::clone(&self.bindings)
    }
    /// Explicit diagnostic materialization of full performed bindings.
    pub(crate) fn snapshot(&self) -> Vec<InvalidationWorkBindingAxes> {
        let buffer = self
            .bindings
            .lock()
            .expect("performed work observation poisoned");
        buffer.entries[..buffer.len]
            .iter()
            .map(|record| {
                record
                    .as_ref()
                    .expect("completed capture slot")
                    .binding
                    .clone()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
