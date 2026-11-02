//! Scoped observation of an actual performed result after product cutover.
use super::RuntimeWorldOperationControlForTest;
use crate::publication::PerformedCompositePublication;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, Weak,
};

type Observer = Box<dyn FnOnce(&PerformedCompositePublication) + Send>;
struct Registration {
    identity: Arc<()>,
    observer: Observer,
}
#[derive(Default)]
pub(super) struct PerformedPublicationSlot {
    armed: AtomicBool,
    pending: Mutex<Option<Registration>>,
}
impl PerformedPublicationSlot {
    fn take(&self) -> Option<Registration> {
        if !self.armed.load(Ordering::Acquire) {
            return None;
        }
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        let registration = pending.take();
        self.armed.store(false, Ordering::Release);
        registration
    }
}

/// Registration custody for this receiver's one pending performed observer.
/// Dropping it disarms only its own still-pending registration.
pub struct RuntimeWorldPerformedPublicationObservationForTest {
    slot: Weak<PerformedPublicationSlot>,
    identity: Arc<()>,
}
impl RuntimeWorldOperationControlForTest {
    /// Observes one actual performed delivery after cutover and outside its
    /// product-cell lock. The callback borrows evidence; it cannot replace the
    /// result, consume delivery or rewrite immutable cancellation facts.
    pub fn observe_next_performed_publication_for_test(
        &self,
        observer: impl FnOnce(&PerformedCompositePublication) + Send + 'static,
    ) -> RuntimeWorldPerformedPublicationObservationForTest {
        let registration = Registration {
            identity: Arc::new(()),
            observer: Box::new(observer),
        };
        let identity = registration.identity.clone();
        {
            let mut pending = self
                .performed_publication
                .pending
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            assert!(pending.is_none(), "one performed observer may be armed");
            *pending = Some(registration);
            self.performed_publication
                .armed
                .store(true, Ordering::Release);
        }
        RuntimeWorldPerformedPublicationObservationForTest {
            slot: Arc::downgrade(&self.performed_publication),
            identity,
        }
    }

    pub(crate) fn observe_performed_publication_for_test(
        &self,
        performed: &PerformedCompositePublication,
    ) {
        if let Some(registration) = self.performed_publication.take() {
            // Invocation and destruction of opaque captures are outside the slot lock.
            (registration.observer)(performed);
        }
    }
}
impl Drop for RuntimeWorldPerformedPublicationObservationForTest {
    fn drop(&mut self) {
        let removed = self.slot.upgrade().and_then(|slot| {
            let mut pending = slot.pending.lock().unwrap_or_else(|e| e.into_inner());
            if pending
                .as_ref()
                .is_some_and(|entry| Arc::ptr_eq(&entry.identity, &self.identity))
            {
                let removed = pending.take();
                slot.armed.store(false, Ordering::Release);
                removed
            } else {
                None
            }
        });
        // A capture may reenter this receiver while being destroyed.
        drop(removed);
    }
}

#[cfg(test)]
#[path = "performed_publication/tests.rs"]
mod tests;
