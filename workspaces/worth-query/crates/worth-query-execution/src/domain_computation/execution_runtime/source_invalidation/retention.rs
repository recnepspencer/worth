use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::WorthQueryInvalidationResourceDenial;

#[derive(Debug)]
pub(super) struct InvalidationRetentionLedger {
    maximum_bytes: u64,
    retained_bytes: AtomicU64,
}

impl InvalidationRetentionLedger {
    pub(super) const fn new(maximum_bytes: u64) -> Self {
        Self {
            maximum_bytes,
            retained_bytes: AtomicU64::new(0),
        }
    }

    pub(super) fn retained_bytes(&self) -> u64 {
        self.retained_bytes.load(Ordering::Acquire)
    }

    pub(super) fn reserve(
        self: &Arc<Self>,
        bytes: u64,
    ) -> Result<RetainedInvalidationCapacity, WorthQueryInvalidationResourceDenial> {
        self.retained_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |retained| {
                retained
                    .checked_add(bytes)
                    .filter(|next| *next <= self.maximum_bytes)
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
    pub(in crate::domain_computation) const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for RetainedInvalidationCapacity {
    fn drop(&mut self) {
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
    fn representable_maximum_does_not_accept_retention_counter_overflow() {
        let ledger = Arc::new(InvalidationRetentionLedger::new(u64::MAX));
        let maximum = ledger.reserve(u64::MAX).unwrap();
        assert!(ledger.reserve(1).is_err());
        assert_eq!(ledger.retained_bytes(), u64::MAX);
        drop(maximum);
        assert_eq!(ledger.retained_bytes(), 0);
    }
}
