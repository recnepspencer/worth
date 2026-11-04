//! Sealed observer of read safe points: a court schedules an external
//! cancellation at a real kernel safe point. It cannot forge admission,
//! substitute rows, or bypass the production meter. Disarmed, a safe point only
//! finds the slot empty.
use std::cell::RefCell;
use std::marker::PhantomData;

thread_local! {
    static OBSERVER: RefCell<Option<Box<dyn FnMut(&str)>>> = const { RefCell::new(None) };
}

/// Owned handle of an armed observer; dropping it, unwinding included, disarms.
pub struct CheckpointObserverForTest {
    _thread_bound: PhantomData<*const ()>,
}

impl Drop for CheckpointObserverForTest {
    fn drop(&mut self) {
        OBSERVER.with(|slot| *slot.borrow_mut() = None);
    }
}

/// Arms `callback` at every safe point on this thread until the handle drops.
pub fn observe_checkpoints_for_test(callback: Box<dyn FnMut(&str)>) -> CheckpointObserverForTest {
    OBSERVER.with(|slot| *slot.borrow_mut() = Some(callback));
    CheckpointObserverForTest {
        _thread_bound: PhantomData,
    }
}

pub(super) fn visit(subject: &str) {
    OBSERVER.with(|slot| {
        if let Some(callback) = slot.borrow_mut().as_mut() {
            callback(subject);
        }
    });
}
