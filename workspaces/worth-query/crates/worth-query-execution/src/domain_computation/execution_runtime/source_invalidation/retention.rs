use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::WorthQueryInvalidationResourceDenial;

#[derive(Debug)]
pub(in crate::domain_computation) struct InvalidationRetentionLedger {
    pub(in crate::domain_computation) oracle_guards: std::sync::atomic::AtomicUsize,
    pub(in crate::domain_computation) oracle_checks: AtomicU64,
    maximum_bytes: u64,
    retained_bytes: AtomicU64,
    pub(super) exhaust_delivery: std::sync::atomic::AtomicBool,
    pub(super) native: super::native_reservation_observation::NativeReservationRecorder,
}

impl InvalidationRetentionLedger {
    pub(super) const fn new(maximum_bytes: u64) -> Self {
        Self {
            maximum_bytes,
            oracle_guards: std::sync::atomic::AtomicUsize::new(0),
            oracle_checks: AtomicU64::new(0),
            retained_bytes: AtomicU64::new(0),
            exhaust_delivery: std::sync::atomic::AtomicBool::new(false),
            native: super::native_reservation_observation::NativeReservationRecorder::new(),
        }
    }

    pub(super) fn install(maximum_bytes: u64) -> Arc<Self> {
        let ledger = Arc::new(Self::new(maximum_bytes));
        ledger.attach_native_observation();
        // Law 2: construction may finish on a worker after resource installation.
        crate::domain_computation::invalidation_equivalence_control::attach_constructed_receiver(
            &ledger,
        );
        ledger
    }
    pub(super) fn attach_native_observation(self: &Arc<Self>) {
        if crate::domain_computation::execution_observation::is_installed() {
            self.native.install();
            let receiver = Arc::downgrade(self);
            crate::domain_computation::execution_observation::register_owned_cleanup(move || {
                if let Some(ledger) = receiver.upgrade() {
                    ledger.native.clear();
                }
            });
        }
    }
    pub(super) fn retained_bytes(&self) -> u64 {
        self.retained_bytes.load(Ordering::Acquire)
    }

    /// Reserves only against capacity whose custody has actually ended.
    /// Pinned or prepared predecessor images still count against the ceiling.
    pub(super) fn reserve(
        self: &Arc<Self>,
        bytes: u64,
    ) -> Result<RetainedInvalidationCapacity, WorthQueryInvalidationResourceDenial> {
        let maximum = self.maximum_bytes;
        self.retained_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |retained| {
                retained.checked_add(bytes).filter(|next| *next <= maximum)
            })
            .map_err(|retained| {
                WorthQueryInvalidationResourceDenial::RetentionCapacityExhausted {
                    requested: bytes,
                    retained,
                    maximum: self.maximum_bytes,
                }
            })?;
        Ok(RetainedInvalidationCapacity {
            ledger: Arc::clone(self),
            bytes,
        })
    }
}

/// Move-only custody of an admitted retained capacity bound. A root's lifetime
/// owns its reservation; rejecting or retiring that root releases it.
#[derive(Debug)]
pub(in crate::domain_computation) struct RetainedInvalidationCapacity {
    ledger: Arc<InvalidationRetentionLedger>,
    bytes: u64,
}

impl RetainedInvalidationCapacity {
    pub(in crate::domain_computation) fn observe_custody(
        &self,
        site: &'static std::panic::Location<'static>,
    ) {
        self.ledger
            .native
            .observe(self as *const Self as usize, self.bytes, site);
    }
    pub(in crate::domain_computation) fn observe_native(&self, kind: super::NativeRetainedKind) {
        self.ledger
            .native
            .retain(self as *const Self as usize, kind, self.bytes);
    }
    pub(in crate::domain_computation) fn observe_consumed(self: &Arc<Self>, runtime: u64) {
        self.ledger.native.observe_consumed(runtime, self);
    }

    pub(in crate::domain_computation) const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for RetainedInvalidationCapacity {
    fn drop(&mut self) {
        self.ledger.native.release(self as *const Self as usize);
        self.ledger
            .retained_bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };

    #[test]
    fn source_bindings_share_capacity_and_rejection_preserves_custody() {
        let resources =
            WorthQueryInvalidationResources::install(WorthQueryInvalidationResourceInstallation {
                maximum_marking_work: 1,
                maximum_preparation_bytes: 1,
                maximum_retained_bytes: 17,
                maximum_retained_positions: 1,
            })
            .unwrap();
        let other_source = resources.clone();
        let first = resources.reserve_retained_capacity(11).unwrap();
        let second = other_source.reserve_retained_capacity(6).unwrap();
        assert!(matches!(
            resources.reserve_retained_capacity(1),
            Err(
                WorthQueryInvalidationResourceDenial::RetentionCapacityExhausted {
                    requested: 1,
                    retained: 17,
                    maximum: 17,
                }
            )
        ));
        assert_eq!(other_source.retained_capacity_bytes(), 17);
        drop(first);
        assert_eq!(resources.retained_capacity_bytes(), 6);
        drop(second);
        assert_eq!(resources.retained_capacity_bytes(), 0);
    }

    #[test]
    fn a_pinned_predecessor_is_not_release_credit() {
        let ledger = Arc::new(InvalidationRetentionLedger::new(10));
        let predecessor = ledger.reserve(10).unwrap();
        assert!(ledger.reserve(10).is_err());
        assert_eq!(ledger.retained_bytes(), 10);
        drop(predecessor);
        let replacement = ledger.reserve(10).unwrap();
        assert_eq!(ledger.retained_bytes(), 10);
        drop(replacement);
        assert_eq!(ledger.retained_bytes(), 0);
    }

    #[test]
    fn representable_maximum_does_not_accept_retention_counter_overflow() {
        let ledger = Arc::new(InvalidationRetentionLedger::new(u64::MAX));
        let maximum = ledger.reserve(u64::MAX).unwrap();
        assert!(ledger.reserve(1).is_err());
        assert_eq!(ledger.retained_bytes(), u64::MAX);
        drop(maximum);
        assert_eq!(ledger.retained_bytes(), 0);
    }
}
