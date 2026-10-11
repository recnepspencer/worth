//! A test-local rendezvous immediately before an envelope's effects begin.
use std::cell::RefCell;

type Observer = Box<dyn FnMut()>;
thread_local! {
    static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
}

struct Restore(Option<Observer>);
impl Drop for Restore {
    fn drop(&mut self) {
        OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

pub(super) fn with_observer<R>(observer: Observer, body: impl FnOnce() -> R) -> R {
    let previous = OBSERVER.with(|slot| slot.replace(Some(observer)));
    let _restore = Restore(previous);
    body()
}

pub(super) fn visit() {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow_mut().as_mut() {
            observer();
        }
    });
}
