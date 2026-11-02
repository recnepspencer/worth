//! Opaque captures close outside the real slot lock; old guards cannot clear replacements.
use super::*;

struct Capture {
    slot: Arc<PerformedPublicationSlot>,
    dropped: Arc<AtomicBool>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        let pending = self
            .slot
            .pending
            .try_lock()
            .expect("capture drops outside slot lock");
        assert!(pending.is_none());
        self.dropped.store(true, Ordering::Release);
    }
}
fn register_capture(
    control: &RuntimeWorldOperationControlForTest,
    dropped: &Arc<AtomicBool>,
) -> RuntimeWorldPerformedPublicationObservationForTest {
    let capture = Capture {
        slot: control.performed_publication.clone(),
        dropped: dropped.clone(),
    };
    control.observe_next_performed_publication_for_test(move |_| drop(capture))
}

#[test]
fn performed_observer_closes_captures_unlocked_and_preserves_replacement_registration() {
    let control = RuntimeWorldOperationControlForTest::default();
    let dropped = Arc::new(AtomicBool::new(false));
    let guard = register_capture(&control, &dropped);
    drop(guard);
    assert!(dropped.load(Ordering::Acquire));
    assert!(!control.performed_publication.armed.load(Ordering::Acquire));

    dropped.store(false, Ordering::Release);
    let consumed = register_capture(&control, &dropped);
    // Exercise the exact take used by dispatch, without inventing a performed receipt.
    drop(control.performed_publication.take().unwrap());
    assert!(dropped.load(Ordering::Acquire));
    let replacement_dropped = Arc::new(AtomicBool::new(false));
    let replacement = register_capture(&control, &replacement_dropped);
    drop(consumed);
    assert!(control.performed_publication.armed.load(Ordering::Acquire));
    assert!(!replacement_dropped.load(Ordering::Acquire));
    drop(replacement);
    assert!(replacement_dropped.load(Ordering::Acquire));
}
