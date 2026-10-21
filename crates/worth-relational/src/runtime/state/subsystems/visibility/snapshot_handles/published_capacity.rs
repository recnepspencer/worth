use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// The accounted admission owner for published snapshot custody and its optional policy.
///
/// Capacity is accounted separately from the registry itself so admission is a
/// nonblocking reservation that never touches a registry lock.
#[derive(Debug)]
pub(crate) struct PublishedSnapshotCapacityOwner {
    maximum_handles: Option<usize>,
    occupied_handles: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PublishedSnapshotReservationDenial {
    Capacity { maximum_handles: usize },
    CounterOverflow,
}

impl PublishedSnapshotCapacityOwner {
    pub(crate) fn new(maximum_handles: Option<usize>) -> Arc<Self> {
        Arc::new(Self {
            maximum_handles,
            occupied_handles: AtomicUsize::new(0),
        })
    }

    pub(crate) fn reserve(
        self: &Arc<Self>,
    ) -> Result<PublishedSnapshotSlotReservation, PublishedSnapshotReservationDenial> {
        let mut occupied = self.occupied_handles.load(Ordering::Acquire);
        loop {
            if let Some(maximum_handles) =
                self.maximum_handles.filter(|maximum| occupied >= *maximum)
            {
                return Err(PublishedSnapshotReservationDenial::Capacity { maximum_handles });
            }
            let next = occupied
                .checked_add(1)
                .ok_or(PublishedSnapshotReservationDenial::CounterOverflow)?;
            match self.occupied_handles.compare_exchange_weak(
                occupied,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(PublishedSnapshotSlotReservation {
                        owner: Arc::clone(self),
                        releases_on_drop: true,
                    });
                }
                Err(observed) => occupied = observed,
            }
        }
    }

    pub(crate) const fn maximum_handles(&self) -> Option<usize> {
        self.maximum_handles
    }

    pub(crate) fn occupied_handles(&self) -> usize {
        self.occupied_handles.load(Ordering::Acquire)
    }

    pub(crate) fn release(&self) {
        let previous = self.occupied_handles.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "published snapshot capacity underflow");
    }
}

#[derive(Debug)]
#[must_use = "published snapshot capacity reservations must be installed or released"]
pub(crate) struct PublishedSnapshotSlotReservation {
    owner: Arc<PublishedSnapshotCapacityOwner>,
    releases_on_drop: bool,
}

impl PublishedSnapshotSlotReservation {
    pub(crate) fn install(mut self) {
        self.releases_on_drop = false;
    }
}

impl Drop for PublishedSnapshotSlotReservation {
    fn drop(&mut self) {
        if self.releases_on_drop {
            self.owner.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_policy_keeps_counting_and_refuses_arithmetic_exhaustion() {
        let owner = PublishedSnapshotCapacityOwner::new(None);
        let held: Vec<_> = (0..600).map(|_| owner.reserve().unwrap()).collect();
        let shared_owner = Arc::clone(&owner);
        assert_eq!(shared_owner.occupied_handles(), held.len());
        drop(held);
        assert_eq!(owner.occupied_handles(), 0);
        owner.occupied_handles.store(usize::MAX, Ordering::Release);
        assert!(matches!(
            owner.reserve(),
            Err(PublishedSnapshotReservationDenial::CounterOverflow)
        ));
        assert_eq!(owner.occupied_handles(), usize::MAX);
        owner.occupied_handles.store(0, Ordering::Release);
        drop(owner.reserve().unwrap());
        assert_eq!(owner.occupied_handles(), 0);
    }

    #[test]
    fn finite_slots_refuse_before_admission_and_refund_on_drop() {
        let owner = PublishedSnapshotCapacityOwner::new(Some(2));
        let first = owner.reserve().unwrap();
        let second = owner.reserve().unwrap();
        assert!(matches!(
            owner.reserve(),
            Err(PublishedSnapshotReservationDenial::Capacity { maximum_handles: 2 })
        ));
        assert_eq!(owner.occupied_handles(), 2);
        drop(first);
        let replacement = owner.reserve().unwrap();
        assert_eq!(owner.occupied_handles(), 2);
        drop((second, replacement));
        assert_eq!(owner.occupied_handles(), 0);
    }
}
