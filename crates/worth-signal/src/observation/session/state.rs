use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge, RetainedStoragePreparationDenial,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::completion::SignalObservationCompletion;
use super::request::{SignalObservationRequest, SignalObservationSurface};

/// Runtime-owned lifecycle state for one managed observation owner.
///
/// Counters and work capture borrow the cloneable gate, but they do not own
/// session generation, admission, or terminal transitions.
#[derive(Debug)]
pub(crate) struct SignalObservationSessionState {
    next_generation: AtomicU64,
    active_generation: Arc<AtomicU64>,
    active_surface_mask: Arc<AtomicU64>,
    default_surface_mask: Arc<AtomicU64>,
    completed_execution_boundaries: Arc<AtomicU64>,
    last_completion: Arc<AtomicU64>,
}

/// Read-only capture gate shared with performed counter/work storage.
#[derive(Debug, Clone)]
pub(crate) struct SignalObservationCaptureGate {
    active_generation: Arc<AtomicU64>,
    active_surface_mask: Arc<AtomicU64>,
    default_surface_mask: Arc<AtomicU64>,
}

impl Default for SignalObservationSessionState {
    fn default() -> Self {
        Self {
            next_generation: AtomicU64::new(0),
            active_generation: Arc::new(AtomicU64::new(0)),
            active_surface_mask: Arc::new(AtomicU64::new(0)),
            default_surface_mask: Arc::new(AtomicU64::new(0)),
            completed_execution_boundaries: Arc::new(AtomicU64::new(0)),
            last_completion: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Default for SignalObservationCaptureGate {
    fn default() -> Self {
        Self {
            active_generation: Arc::new(AtomicU64::new(0)),
            active_surface_mask: Arc::new(AtomicU64::new(0)),
            default_surface_mask: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl SignalObservationSessionState {
    pub(crate) fn initial_heap_charge(
    ) -> Result<RetainedStorageCharge, RetainedStoragePreparationDenial> {
        arc_allocation_charge::<AtomicU64>()?.checked_mul(5)
    }

    pub(crate) fn capture_gate(&self) -> SignalObservationCaptureGate {
        SignalObservationCaptureGate {
            active_generation: Arc::clone(&self.active_generation),
            active_surface_mask: Arc::clone(&self.active_surface_mask),
            default_surface_mask: Arc::clone(&self.default_surface_mask),
        }
    }

    pub(crate) fn begin(&self, request: SignalObservationRequest) -> u64 {
        let generation = self
            .next_generation
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        self.active_surface_mask
            .store(u64::from(request.mask()), Ordering::Release);
        self.completed_execution_boundaries
            .store(0, Ordering::Release);
        self.last_completion.store(0, Ordering::Release);
        self.active_generation.store(generation, Ordering::Release);
        generation
    }

    pub(crate) fn active_generation(&self) -> u64 {
        self.active_generation.load(Ordering::Acquire)
    }

    pub(crate) fn active_request(&self) -> SignalObservationRequest {
        SignalObservationRequest::from_mask(self.active_surface_mask.load(Ordering::Acquire) as u8)
    }

    pub(crate) fn liveness(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.active_generation)
    }

    pub(crate) fn finish(&self, generation: u64) -> bool {
        let finished = self
            .active_generation
            .compare_exchange(generation, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        if finished {
            self.completed_execution_boundaries
                .store(0, Ordering::Release);
            self.active_surface_mask.store(
                self.default_surface_mask.load(Ordering::Acquire),
                Ordering::Release,
            );
        }
        finished
    }

    pub(crate) fn interrupt(&self) -> bool {
        let generation = self.active_generation();
        generation != 0 && self.finish(generation)
    }

    pub(crate) fn record_completed_execution_boundary(&self) {
        if self.active_generation() != 0 {
            self.completed_execution_boundaries
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn completed_execution_boundaries(&self) -> u64 {
        self.completed_execution_boundaries.load(Ordering::Acquire)
    }

    pub(crate) fn shared_completed_execution_boundaries(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.completed_execution_boundaries)
    }

    pub(crate) fn shared_last_completion(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.last_completion)
    }

    pub(crate) fn record_completion(&self, completion: SignalObservationCompletion) {
        self.last_completion
            .store(u64::from(completion.code()), Ordering::Release);
    }

    pub(crate) fn last_completion(&self) -> Option<SignalObservationCompletion> {
        SignalObservationCompletion::from_code(self.last_completion.load(Ordering::Acquire) as u8)
    }

    pub(crate) fn set_default_surface_mask(&self, surface_mask: u8) {
        self.default_surface_mask
            .store(u64::from(surface_mask), Ordering::Release);
        if self.active_generation() == 0 {
            self.active_surface_mask
                .store(u64::from(surface_mask), Ordering::Release);
        }
    }
}

impl SignalObservationCaptureGate {
    pub(crate) fn captures(&self, surface: SignalObservationSurface) -> bool {
        let mask = if self.active_generation.load(Ordering::Acquire) == 0 {
            self.default_surface_mask.load(Ordering::Acquire)
        } else {
            self.active_surface_mask.load(Ordering::Acquire)
        };
        mask & u64::from(surface.bit()) != 0
    }
}
