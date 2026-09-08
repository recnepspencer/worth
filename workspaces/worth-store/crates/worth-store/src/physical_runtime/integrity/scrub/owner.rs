use super::{cancellation::ScrubRegistration, PhysicalIntegrityScrubRequestDenial};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, Weak,
};

/// Lives before the serving runtime's subordinate owners. Drop cancels every
/// handle, then joins the single in-flight window before media/pool teardown.
pub(in crate::physical_runtime) struct PhysicalIntegrityScrubOwner {
    registrations: Mutex<Vec<Weak<ScrubRegistration>>>,
    next_identity: AtomicU64,
    pub(super) window: Arc<Mutex<()>>,
}

impl PhysicalIntegrityScrubOwner {
    pub(in crate::physical_runtime) fn new() -> Self {
        Self {
            registrations: Mutex::new(Vec::new()),
            next_identity: AtomicU64::new(1),
            window: Arc::new(Mutex::new(())),
        }
    }
    pub(super) fn register(
        &self,
    ) -> Result<(u64, Arc<ScrubRegistration>), PhysicalIntegrityScrubRequestDenial> {
        let mut registrations = self
            .registrations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registrations.retain(|entry| {
            entry
                .upgrade()
                .is_some_and(|registration| !registration.finished.load(Ordering::Acquire))
        });
        if registrations.len() >= 16 {
            return Err(PhysicalIntegrityScrubRequestDenial::ActiveHandleLimitExceeded);
        }
        let identity = self
            .next_identity
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |identity| {
                identity.checked_add(1)
            })
            .map_err(|_| PhysicalIntegrityScrubRequestDenial::ActiveHandleLimitExceeded)?;
        let registration = Arc::new(ScrubRegistration {
            cancelled: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            finished: AtomicBool::new(false),
        });
        registrations.push(Arc::downgrade(&registration));
        Ok((identity, registration))
    }
}

impl Drop for PhysicalIntegrityScrubOwner {
    fn drop(&mut self) {
        for registration in self
            .registrations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(Weak::upgrade)
        {
            registration.closed.store(true, Ordering::Release);
        }
        let _settled_window = self
            .window
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}
