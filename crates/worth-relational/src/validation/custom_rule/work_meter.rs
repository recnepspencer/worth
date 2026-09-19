use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct CustomInvariantWorkMeter {
    maximum: NonZeroU64,
    consumed: Arc<AtomicU64>,
    exhausted: Arc<AtomicBool>,
}

impl CustomInvariantWorkMeter {
    pub(crate) fn new(maximum: NonZeroU64) -> Self {
        Self {
            maximum,
            consumed: Arc::new(AtomicU64::new(0)),
            exhausted: Arc::new(AtomicBool::new(false)),
        }
    }
    pub(crate) fn try_charge(&self, units: usize) -> bool {
        if self.exceeded() {
            return false;
        }
        let units = u64::try_from(units).unwrap_or(u64::MAX);
        if self
            .consumed
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current
                    .checked_add(units)
                    .filter(|next| *next <= self.maximum.get())
            })
            .is_err()
        {
            self.mark_exceeded();
            return false;
        }
        true
    }
    pub(crate) fn consumed(&self) -> NonZeroU64 {
        NonZeroU64::new(self.consumed.load(Ordering::Relaxed).max(1)).unwrap()
    }
    pub(crate) fn exceeded(&self) -> bool {
        self.exhausted.load(Ordering::Relaxed)
    }
    pub(crate) fn mark_exceeded(&self) {
        self.exhausted.store(true, Ordering::Relaxed);
    }
}
