use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::WorthQueryProductActivationDenial;

pub(super) struct ActivationCapacity {
    maximum: Option<usize>,
    live: AtomicUsize,
}

impl ActivationCapacity {
    pub(super) fn new(maximum: Option<usize>) -> Arc<Self> {
        Arc::new(Self {
            maximum,
            live: AtomicUsize::new(0),
        })
    }

    pub(super) fn reserve(
        self: &Arc<Self>,
    ) -> Result<ActivationCapacityReservation, WorthQueryProductActivationDenial> {
        self.live
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |live| {
                if self.maximum.is_some_and(|maximum| live >= maximum) {
                    None
                } else {
                    live.checked_add(1)
                }
            })
            // fetch_update returns the value at the failed atomic reservation,
            // rather than a later load that could race with a release.
            .map_err(|occupancy| {
                if let Some(limit) = self.maximum.filter(|maximum| occupancy >= *maximum) {
                    debug_assert_eq!(
                        occupancy, limit,
                        "activation capacity occupancy exceeds its installed limit"
                    );
                    WorthQueryProductActivationDenial::CapacityExhausted { limit, occupancy }
                } else {
                    WorthQueryProductActivationDenial::OccupancyOverflow
                }
            })?;
        Ok(ActivationCapacityReservation(Arc::clone(self)))
    }
}

/// Follows the gate allocation, including a gate held after its branch retires.
pub(super) struct ActivationCapacityReservation(Arc<ActivationCapacity>);

impl Drop for ActivationCapacityReservation {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_activation_policy_accounts_refunds_and_refuses_overflow() {
        let owner = ActivationCapacity::new(None);
        let held: Vec<_> = (0..600).map(|_| owner.reserve().unwrap()).collect();
        assert_eq!(owner.live.load(Ordering::Acquire), held.len());
        drop(held);
        assert_eq!(owner.live.load(Ordering::Acquire), 0);
        owner.live.store(usize::MAX, Ordering::Release);
        assert_eq!(
            owner.reserve().err(),
            Some(WorthQueryProductActivationDenial::OccupancyOverflow)
        );
        assert_eq!(owner.live.load(Ordering::Acquire), usize::MAX);
        owner.live.store(0, Ordering::Release);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "activation capacity occupancy exceeds its installed limit")]
    fn activation_capacity_desynchronization_is_loud_in_debug_builds() {
        let owner = ActivationCapacity::new(Some(1));
        owner.live.store(2, Ordering::Release);
        let _ = owner.reserve();
    }
}
