use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(super) struct ScrubRegistration {
    pub(super) cancelled: AtomicBool,
    pub(super) closed: AtomicBool,
    pub(super) finished: AtomicBool,
}

#[derive(Clone)]
pub struct PhysicalIntegrityScrubCancellation {
    pub(super) registration: Arc<ScrubRegistration>,
}

impl PhysicalIntegrityScrubCancellation {
    pub fn cancel(&self) {
        self.registration.cancelled.store(true, Ordering::Release);
    }
}
