use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
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

/// Read-only view of the query result buffers this runtime holds.
///
/// Get one from `result_buffer_observer` and call `observe` for a
/// point-in-time reading. Observing grants nothing and changes nothing.
#[derive(Clone)]
pub struct WorthQueryApplicationResultBufferObserver {
    state: Arc<ResultBufferRegistryState>,
}

/// Point-in-time reading of query result buffers: how many are active, the
/// bytes they retain, the largest size any buffer reached, and the largest
/// claim that was refused.
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
        }
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
        if acquire(&self.registry.state.retained_bytes, bytes).is_err() {
            self.record_rejected(usize::MAX);
            return Err(());
        }
        self.peak_bytes = self.peak_bytes.max(claimed);
        self.registry
            .state
            .peak_observed_bytes
            .fetch_max(claimed, Ordering::AcqRel);
        self.retained_bytes = claimed;
        Ok(())
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
        self.released = true;
    }
}

impl Drop for WorthQueryApplicationResultBufferReservation {
    fn drop(&mut self) {
        self.release_owned_bytes();
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
}
