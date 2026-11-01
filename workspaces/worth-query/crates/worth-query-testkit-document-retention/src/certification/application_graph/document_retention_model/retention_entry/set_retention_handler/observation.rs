//! Installation transports only diagnostic placement; the actual handler owns every count.
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Default)]
pub(super) struct Contacts {
    decisions: AtomicUsize,
    candidates: AtomicUsize,
}

thread_local! {
    static INSTALLING: RefCell<Option<Arc<Contacts>>> = const { RefCell::new(None) };
}

pub struct RetentionHandlerObservationForTest(Arc<Contacts>);

impl RetentionHandlerObservationForTest {
    pub fn snapshot(&self) -> (usize, usize) {
        (
            self.0.decisions.load(Ordering::Relaxed),
            self.0.candidates.load(Ordering::Relaxed),
        )
    }
}

struct Placement(Option<Arc<Contacts>>);
impl Drop for Placement {
    fn drop(&mut self) {
        INSTALLING.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}

pub(super) fn install<T>(build: impl FnOnce() -> T) -> (T, RetentionHandlerObservationForTest) {
    let contacts = Arc::new(Contacts::default());
    let placement = Placement(INSTALLING.with(|slot| slot.replace(Some(Arc::clone(&contacts)))));
    let installed = build();
    drop(placement);
    (installed, RetentionHandlerObservationForTest(contacts))
}

pub(super) fn capture() -> Option<Arc<Contacts>> {
    INSTALLING.with(|slot| slot.borrow().clone())
}

impl Contacts {
    pub(super) fn decision(&self) {
        self.decisions.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn candidate(&self) {
        self.candidates.fetch_add(1, Ordering::Relaxed);
    }
}
