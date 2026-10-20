use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::RuntimeWorldOwnerIdentity;

use super::RetentionObligationDenial;

/// One owner's bounded count of reserved and live observation obligations.
/// Cloned product observations share their obligation and its single charge.
#[derive(Debug)]
pub(super) struct ObservationCapacity {
    owner: RuntimeWorldOwnerIdentity,
    maximum: Option<usize>,
    active: AtomicUsize,
}

impl ObservationCapacity {
    pub(super) fn new(
        owner: RuntimeWorldOwnerIdentity,
        limit: RuntimeWorldBudgetLimit,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner,
            maximum: limit.get(),
            active: AtomicUsize::new(0),
        })
    }

    pub(super) fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }

    pub(super) fn reserve(
        self: &Arc<Self>,
    ) -> Result<ReservedObservationCapacity, RetentionObligationDenial> {
        self.active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                if self.maximum.is_some_and(|maximum| active >= maximum) {
                    None
                } else {
                    active.checked_add(1)
                }
            })
            .map_err(|_| RetentionObligationDenial::ObservationCapacityExhausted)?;
        Ok(ReservedObservationCapacity {
            capacity: Arc::clone(self),
        })
    }
}

/// Reserved before a fork and transferred into its returned observation.
#[derive(Debug)]
pub(crate) struct ReservedObservationCapacity {
    capacity: Arc<ObservationCapacity>,
}

impl ReservedObservationCapacity {
    pub(crate) fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.capacity.owner
    }
}

impl Drop for ReservedObservationCapacity {
    fn drop(&mut self) {
        self.capacity.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_policy_still_refuses_counter_overflow_without_changing_custody() {
        let authority =
            crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract::new().unwrap();
        let capacity = ObservationCapacity::new(
            authority.owner_identity(),
            crate::budget::RuntimeWorldBudgets::without_policy_limits().active_observations(),
        );
        capacity.active.store(usize::MAX, Ordering::Release);
        assert!(matches!(
            capacity.reserve(),
            Err(RetentionObligationDenial::ObservationCapacityExhausted)
        ));
        assert_eq!(capacity.active(), usize::MAX);
        capacity.active.store(0, Ordering::Release);
        let reservation = capacity.reserve().unwrap();
        assert_eq!(capacity.active(), 1);
        drop(reservation);
        assert_eq!(capacity.active(), 0);
    }
}
