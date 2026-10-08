use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::super::batch::{
    WorthQueryApplicationQueryBatchAdmission, WorthQueryApplicationQueryBatchMemory,
};
use super::lifecycle_count::{acquire, release};

#[derive(Default)]
struct ResultBufferRegistryState {
    active_buffers: AtomicUsize,
    retained_bytes: AtomicUsize,
    peak_observed_bytes: AtomicUsize,
    peak_rejected_bytes: AtomicUsize,
}

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationResultBufferRegistry {
    state: Arc<ResultBufferRegistryState>,
}

/// Read-only view of query result buffers and disclosed source custody.
///
/// Get one from `result_buffer_observer` and call `observe` for a
/// point-in-time reading. Observing grants nothing and changes nothing.
#[derive(Clone)]
pub struct WorthQueryApplicationResultBufferObserver {
    state: Arc<ResultBufferRegistryState>,
}

/// Point-in-time reading: active buffers, bytes held by buffers or disclosed
/// sources, the largest buffer claim, and the largest refused claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationResultBufferObservation {
    active_buffers: usize,
    retained_bytes: usize,
    peak_observed_bytes: usize,
    peak_rejected_bytes: usize,
}

/// Result buffer use for one query read: its byte limit, the most bytes it
/// held, and whether it was released.
///
/// Read it from an access receipt's `result_buffer()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationResultBufferEvidence {
    limit_bytes: usize,
    peak_bytes: usize,
    released: bool,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationResultBufferReservation
{
    registry: WorthQueryApplicationResultBufferRegistry,
    limit_bytes: usize,
    retained_bytes: usize,
    peak_bytes: usize,
    released: bool,
    batch: Option<WorthQueryApplicationQueryBatchAdmission>,
    batch_buffer: Option<WorthQueryApplicationQueryBatchMemory>,
}

/// Bytes admitted by a query result buffer and retained by a disclosed source.
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryRetainedSourceCharge
{
    state: Arc<ResultBufferRegistryState>,
    bytes: usize,
    _batch: Option<WorthQueryApplicationQueryBatchMemory>,
}

impl WorthQueryApplicationResultBufferRegistry {
    pub(in crate::domain_computation::primary_graph) fn observer(
        &self,
    ) -> WorthQueryApplicationResultBufferObserver {
        WorthQueryApplicationResultBufferObserver {
            state: Arc::clone(&self.state),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn reserve(
        &self,
        limit_bytes: usize,
    ) -> WorthQueryApplicationResultBufferReservation {
        acquire(&self.state.active_buffers, 1)
            .expect("live application-query result-buffer count cannot overflow");
        WorthQueryApplicationResultBufferReservation {
            registry: self.clone(),
            limit_bytes,
            retained_bytes: 0,
            peak_bytes: 0,
            released: false,
            batch: None,
            batch_buffer: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn reserve_in_batch(
        &self,
        limit_bytes: usize,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> WorthQueryApplicationResultBufferReservation {
        let mut reservation = self.reserve(limit_bytes);
        reservation.batch = Some(batch.retained_meter());
        reservation.batch_buffer = Some(
            batch
                .claim_memory(0)
                .expect("a zero claim cannot exceed an admitted allowance"),
        );
        reservation
    }
}

impl WorthQueryApplicationResultBufferObserver {
    pub fn observe(&self) -> WorthQueryApplicationResultBufferObservation {
        WorthQueryApplicationResultBufferObservation {
            active_buffers: self.state.active_buffers.load(Ordering::Acquire),
            retained_bytes: self.state.retained_bytes.load(Ordering::Acquire),
            peak_observed_bytes: self.state.peak_observed_bytes.load(Ordering::Acquire),
            peak_rejected_bytes: self.state.peak_rejected_bytes.load(Ordering::Acquire),
        }
    }
}

impl WorthQueryApplicationResultBufferObservation {
    pub const fn active_buffers(self) -> usize {
        self.active_buffers
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub const fn peak_observed_bytes(self) -> usize {
        self.peak_observed_bytes
    }

    pub const fn peak_rejected_bytes(self) -> usize {
        self.peak_rejected_bytes
    }
}

impl WorthQueryApplicationResultBufferEvidence {
    pub const fn limit_bytes(self) -> usize {
        self.limit_bytes
    }

    pub const fn peak_bytes(self) -> usize {
        self.peak_bytes
    }

    pub const fn released(self) -> bool {
        self.released
    }
}

impl WorthQueryApplicationResultBufferReservation {
    pub(in crate::domain_computation::primary_graph::application_query) fn claim(
        &mut self,
        bytes: usize,
    ) -> Result<(), ()> {
        let Some(claimed) = self.retained_bytes.checked_add(bytes) else {
            self.record_rejected(usize::MAX);
            return Err(());
        };
        if claimed > self.limit_bytes {
            self.record_rejected(claimed);
            return Err(());
        }
        if let Some(batch) = &mut self.batch_buffer {
            batch.grow(bytes).map_err(|_| ())?;
        }
        if acquire(&self.registry.state.retained_bytes, bytes).is_err() {
            if let Some(batch) = &mut self.batch_buffer {
                batch.release(bytes);
            }
            self.record_rejected(usize::MAX);
            return Err(());
        }
        self.peak_bytes = self.peak_bytes.max(claimed);
        self.registry
            .state
            .peak_observed_bytes
            .fetch_max(claimed, Ordering::AcqRel);
        self.retained_bytes += bytes;
        Ok(())
    }

    /// Disclosed-source custody is retention owned by the source, not result
    /// bytes. It is recorded on the registry's retained ledger until the source
    /// drops and never consumes this read's declared inline-result limit, which
    /// admission already checked against the estimated result.
    pub(in crate::domain_computation::primary_graph::application_query) fn claim_retained_source(
        &self,
        bytes: usize,
    ) -> Result<WorthQueryRetainedSourceCharge, ()> {
        let batch = self
            .batch
            .as_ref()
            .map(|batch| batch.claim_memory(bytes))
            .transpose()
            .map_err(|_| ())?;
        if acquire(&self.registry.state.retained_bytes, bytes).is_err() {
            self.record_rejected(usize::MAX);
            return Err(());
        }
        Ok(WorthQueryRetainedSourceCharge {
            state: Arc::clone(&self.registry.state),
            bytes,
            _batch: batch,
        })
    }

    fn record_rejected(&self, bytes: usize) {
        self.registry
            .state
            .peak_rejected_bytes
            .fetch_max(bytes, Ordering::AcqRel);
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn release_temporary(
        &mut self,
        bytes: usize,
    ) {
        assert!(
            bytes <= self.retained_bytes,
            "result-buffer temporary release cannot exceed claimed bytes"
        );
        release(&self.registry.state.retained_bytes, bytes)
            .expect("global result-buffer retention cannot underflow");
        self.retained_bytes -= bytes;
        if let Some(batch) = &mut self.batch_buffer {
            batch.release(bytes);
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn verify_retained(
        &self,
        bytes: usize,
    ) -> Result<(), ()> {
        (bytes == self.retained_bytes).then_some(()).ok_or(())
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn release(
        mut self,
    ) -> WorthQueryApplicationResultBufferEvidence {
        self.release_owned_bytes();
        WorthQueryApplicationResultBufferEvidence {
            limit_bytes: self.limit_bytes,
            peak_bytes: self.peak_bytes,
            released: self.released,
        }
    }

    fn release_owned_bytes(&mut self) {
        if self.released {
            return;
        }
        release(&self.registry.state.retained_bytes, self.retained_bytes)
            .expect("global result-buffer retention cannot underflow");
        release(&self.registry.state.active_buffers, 1)
            .expect("live application-query result-buffer count cannot underflow");
        self.retained_bytes = 0;
        self.batch_buffer = None;
        self.released = true;
    }
}

impl Drop for WorthQueryApplicationResultBufferReservation {
    fn drop(&mut self) {
        self.release_owned_bytes();
    }
}

impl Drop for WorthQueryRetainedSourceCharge {
    fn drop(&mut self) {
        release(&self.state.retained_bytes, self.bytes)
            .expect("disclosed source retention cannot underflow");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_retention_overflow_is_rejected_without_wrapping_or_local_claim() {
        let registry = WorthQueryApplicationResultBufferRegistry::default();
        registry
            .state
            .retained_bytes
            .store(usize::MAX, Ordering::Release);
        let mut reservation = registry.reserve(8);

        assert_eq!(reservation.claim(1), Err(()));
        assert_eq!(reservation.retained_bytes, 0);
        let observed = registry.observer().observe();
        assert_eq!(observed.retained_bytes(), usize::MAX);
        assert_eq!(observed.peak_observed_bytes(), 0);
        assert_eq!(observed.peak_rejected_bytes(), usize::MAX);

        drop(reservation);
        assert_eq!(registry.observer().observe().active_buffers(), 0);
    }

    #[test]
    fn disclosed_source_outlives_result_buffer_without_losing_its_charge() {
        let registry = WorthQueryApplicationResultBufferRegistry::default();
        let mut reservation = registry.reserve(64);
        reservation.claim(13).unwrap();
        let charge = reservation.claim_retained_source(19).unwrap();
        assert_eq!(registry.observer().observe().retained_bytes(), 32);

        let receipt = reservation.release();
        assert!(receipt.released());
        let observed = registry.observer().observe();
        assert_eq!(observed.active_buffers(), 0);
        assert_eq!(observed.retained_bytes(), 19);
        assert_eq!(receipt.peak_bytes(), 13);

        drop(charge);
        assert_eq!(registry.observer().observe().retained_bytes(), 0);
    }

    #[test]
    fn disclosed_custody_never_consumes_the_inline_result_limit() {
        let registry = WorthQueryApplicationResultBufferRegistry::default();
        let mut reservation = registry.reserve(128);
        reservation.claim(128).unwrap();
        let scope = reservation.claim_retained_source(400).unwrap();
        let parameters = reservation.claim_retained_source(500).unwrap();
        assert_eq!(reservation.claim(1), Err(()));
        assert_eq!(registry.observer().observe().retained_bytes(), 1_028);
        assert_eq!(registry.observer().observe().peak_rejected_bytes(), 129);
        let receipt = reservation.release();
        assert_eq!(receipt.peak_bytes(), 128);
        assert_eq!(registry.observer().observe().retained_bytes(), 900);
        drop((scope, parameters));
        assert_eq!(registry.observer().observe().retained_bytes(), 0);
    }

    #[test]
    fn disclosed_custody_rejects_ledger_overflow_without_a_charge() {
        let registry = WorthQueryApplicationResultBufferRegistry::default();
        registry
            .state
            .retained_bytes
            .store(usize::MAX, Ordering::Release);
        let reservation = registry.reserve(8);
        assert!(reservation.claim_retained_source(1).is_err());
        assert_eq!(registry.observer().observe().retained_bytes(), usize::MAX);
    }
}
