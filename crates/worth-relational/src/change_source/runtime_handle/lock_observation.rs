//! An owned passive listener for one shared runtime handle's acquisitions.
use std::sync::{Arc, Mutex, OnceLock, Weak};
#[derive(Default)]
pub(super) struct LockObservationSlot(OnceLock<Mutex<Weak<Listener>>>);
struct Listener(Box<dyn Fn() + Send + Sync>);
/// Owns observation of the handle and its clones, without granting runtime access.
#[must_use]
pub struct RelationalRuntimeLockObservationForTest {
    _listener: Arc<Listener>,
}
impl LockObservationSlot {
    pub(super) fn install(
        &self,
        acquired: impl Fn() + Send + Sync + 'static,
    ) -> RelationalRuntimeLockObservationForTest {
        let listener = Arc::new(Listener(Box::new(acquired)));
        let mut slot = self
            .0
            .get_or_init(|| Mutex::new(Weak::new()))
            .lock()
            .expect("lock observer is available");
        assert!(
            slot.upgrade().is_none(),
            "this runtime handle already has a lock observer"
        );
        *slot = Arc::downgrade(&listener);
        RelationalRuntimeLockObservationForTest {
            _listener: listener,
        }
    }
    pub(super) fn record(&self) {
        if let Some(slot) = self.0.get() {
            let listener = slot.lock().expect("lock observer is available").upgrade();
            if let Some(listener) = listener {
                (listener.0)();
            }
        }
    }
}
