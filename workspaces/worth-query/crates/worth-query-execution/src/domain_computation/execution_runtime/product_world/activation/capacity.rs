use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::WorthQueryProductActivationDenial;

pub(super) struct ActivationCapacity {
    maximum: usize,
    live: AtomicUsize,
}

impl ActivationCapacity {
    pub(super) fn new(maximum: usize) -> Arc<Self> {
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
                (live < self.maximum).then(|| live + 1)
            })
            .map_err(|_| WorthQueryProductActivationDenial::CapacityExhausted)?;
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
