//! Scoped fixture placement hands existing read observations to the real rule.
use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
};
use worth_query_host::facade::application_invariants::WorthQueryInvariantProjectionWork as Work;
pub(super) type Observer = Arc<dyn Fn(bool, Work) + Send + Sync>;
thread_local! { static PLACEMENT: RefCell<Option<Observer>> = const { RefCell::new(None) }; }
pub(super) fn current() -> Option<Observer> {
    PLACEMENT.with(|slot| slot.borrow().clone())
}
pub(super) struct Capture {
    samples: Arc<Mutex<Vec<(bool, Work)>>>,
    previous: Option<Observer>,
    _thread: PhantomData<Rc<()>>,
}
impl Capture {
    pub fn place() -> Self {
        let samples = Arc::new(Mutex::new(Vec::new()));
        let output = Arc::clone(&samples);
        let callback: Observer =
            Arc::new(move |evaluated, reads| output.lock().unwrap().push((evaluated, reads)));
        let previous = PLACEMENT.with(|slot| slot.replace(Some(callback)));
        Self {
            samples,
            previous,
            _thread: PhantomData,
        }
    }
    pub fn take(&self) -> Vec<(bool, Work)> {
        std::mem::take(&mut *self.samples.lock().unwrap())
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        PLACEMENT.with(|slot| {
            slot.replace(self.previous.take());
        });
    }
}
