//! A synchronous test rendezvous at the maintenance payment boundary.
use std::cell::RefCell;

thread_local! {
    static RENDEZVOUS: RefCell<Option<Box<dyn FnMut()>>> = RefCell::new(None);
}

pub(crate) fn at_next_charges(action: impl FnMut() + 'static) -> Guard {
    RENDEZVOUS.with(|slot| assert!(slot.replace(Some(Box::new(action))).is_none()));
    Guard
}

pub(crate) struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        RENDEZVOUS.with(|slot| {
            slot.replace(None);
        });
    }
}

pub(super) fn meet() {
    RENDEZVOUS.with(|slot| {
        if let Some(action) = slot.borrow_mut().as_mut() {
            action();
        }
    });
}
