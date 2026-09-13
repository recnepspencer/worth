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
    maximum: usize,
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
                (active < self.maximum).then(|| active + 1)
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
