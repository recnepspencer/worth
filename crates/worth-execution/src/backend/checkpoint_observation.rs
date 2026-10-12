//! Owned thread-local observation of charged kernel safe points.
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
// Unarmed consultation pays one thread-local borrow; no listener is allocated.
thread_local! { static LISTENER: RefCell<Option<Box<dyn FnMut()>>> = RefCell::new(None); }
/// Owns a passive listener; it may cancel an existing request, never forge a stop.
#[doc(hidden)]
pub struct ExecutionCheckpointObservationForTest {
    previous: Option<Box<dyn FnMut()>>,
    _thread: PhantomData<Rc<()>>,
}
impl ExecutionCheckpointObservationForTest {
    #[doc(hidden)]
    pub fn install_on_this_thread_for_test(listener: impl FnMut() + 'static) -> Self {
        Self {
            previous: LISTENER.replace(Some(Box::new(listener))),
            _thread: PhantomData,
        }
    }
}
impl Drop for ExecutionCheckpointObservationForTest {
    fn drop(&mut self) {
        LISTENER.replace(self.previous.take());
    }
}
pub(super) fn record(units: u64) {
    if units > 0 {
        LISTENER.with(|slot| {
            if let Some(listener) = slot.borrow_mut().as_mut() {
                listener();
            }
        });
    }
}
