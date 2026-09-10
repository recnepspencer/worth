use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use super::{WorthQueryExecutionCapacityPort, WorthQueryExecutionCapacityReservation};

/// Provider-owned bounded attempt capacity for providers whose physical
/// admission is honestly represented by one concurrent-attempt ceiling.
pub struct WorthQueryFixedExecutionCapacity {
    identity: Arc<str>,
    concurrent_attempt_limit: usize,
    active_attempts: Arc<AtomicUsize>,
}

impl WorthQueryFixedExecutionCapacity {
    pub fn mint(family: &str, concurrent_attempt_limit: usize) -> Option<Self> {
        static NEXT_CAPACITY_SUBJECT: AtomicU64 = AtomicU64::new(1);
        let ordinal = NEXT_CAPACITY_SUBJECT.fetch_add(1, Ordering::Relaxed);
        Self::new(
            Arc::<str>::from(format!("{family}:{ordinal}")),
            concurrent_attempt_limit,
        )
    }

    pub fn new(identity: impl Into<Arc<str>>, concurrent_attempt_limit: usize) -> Option<Self> {
        let identity = identity.into();
        (!identity.trim().is_empty() && concurrent_attempt_limit > 0).then(|| Self {
            identity,
            concurrent_attempt_limit,
            active_attempts: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Current reservations held by admitted execution attempts.
    pub fn active_attempts(&self) -> usize {
        self.active_attempts.load(Ordering::Acquire)
    }
}

impl WorthQueryExecutionCapacityPort for WorthQueryFixedExecutionCapacity {
    fn capacity_subject_identity(&self) -> &str {
        &self.identity
    }

    fn try_reserve(&self) -> Option<Box<dyn WorthQueryExecutionCapacityReservation>> {
        self.active_attempts
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.concurrent_attempt_limit).then_some(active + 1)
            })
            .ok()?;
        Some(Box::new(WorthQueryFixedExecutionCapacityReservation {
            active_attempts: Arc::clone(&self.active_attempts),
        }))
    }
}

struct WorthQueryFixedExecutionCapacityReservation {
    active_attempts: Arc<AtomicUsize>,
}

impl Drop for WorthQueryFixedExecutionCapacityReservation {
    fn drop(&mut self) {
        let prior = self.active_attempts.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prior > 0, "capacity reservations release exactly once");
    }
}
