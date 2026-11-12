//! Observation after an actual Continued acknowledgement; it grants no proof.
use super::RuntimeWorldOperationControlForTest;
use std::sync::{Arc, Mutex, Weak};
struct Registration {
    identity: Arc<()>,
    observer: Box<dyn FnOnce() + Send>,
}
#[derive(Default)]
pub(super) struct RecoveredDispositionSlot {
    pending: Mutex<Option<Registration>>,
}
pub struct RuntimeWorldRecoveredDispositionObservationForTest {
    slot: Weak<RecoveredDispositionSlot>,
    identity: Arc<()>,
}
impl RuntimeWorldOperationControlForTest {
    pub fn observe_recovered_continued_for_test(
        &self,
        observer: impl FnOnce() + Send + 'static,
    ) -> RuntimeWorldRecoveredDispositionObservationForTest {
        let identity = Arc::new(());
        let registration = Registration {
            identity: identity.clone(),
            observer: Box::new(observer),
        };
        let mut pending = self
            .recovered_disposition
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one recovered disposition observation per receiver"
        );
        *pending = Some(registration);
        RuntimeWorldRecoveredDispositionObservationForTest {
            slot: Arc::downgrade(&self.recovered_disposition),
            identity,
        }
    }
    pub(crate) fn recovered_continued_for_test(&self) {
        let registration = self
            .recovered_disposition
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(registration) = registration {
            (registration.observer)();
        }
    }
}
impl Drop for RuntimeWorldRecoveredDispositionObservationForTest {
    fn drop(&mut self) {
        if let Some(slot) = self.slot.upgrade() {
            let registration = {
                let mut pending = slot.pending.lock().unwrap_or_else(|e| e.into_inner());
                if pending
                    .as_ref()
                    .is_some_and(|registration| Arc::ptr_eq(&registration.identity, &self.identity))
                {
                    pending.take()
                } else {
                    None
                }
            };
            drop(registration);
        }
    }
}
