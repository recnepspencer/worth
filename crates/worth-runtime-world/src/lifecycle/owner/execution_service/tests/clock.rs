use super::*;

#[derive(Clone)]
pub(super) struct MutableClock {
    ticks: Arc<AtomicU64>,
}

impl MutableClock {
    pub(super) fn new(ticks: u64) -> Self {
        Self {
            ticks: Arc::new(AtomicU64::new(ticks)),
        }
    }

    pub(super) fn set(&self, ticks: u64) {
        self.ticks.store(ticks, Ordering::Release);
    }
}

impl RuntimeWorldClockSource for MutableClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(self.ticks.load(Ordering::Acquire))
    }
}
