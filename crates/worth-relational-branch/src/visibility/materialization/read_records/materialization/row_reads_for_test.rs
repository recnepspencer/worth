//! Recorder of native entity materialization, scoped to the calling court thread.
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static ARMED_RECORDERS: AtomicUsize = AtomicUsize::new(0);
thread_local! { static ACTIVE: RefCell<Option<Rc<std::cell::Cell<usize>>>> = const { RefCell::new(None) }; }
pub struct RowReadsForTest {
    count: Rc<std::cell::Cell<usize>>,
    previous: Option<Rc<std::cell::Cell<usize>>>,
}
impl RowReadsForTest {
    pub fn install_on_this_thread() -> Self {
        let count = Rc::new(std::cell::Cell::new(0));
        let previous = ACTIVE.replace(Some(Rc::clone(&count)));
        ARMED_RECORDERS.fetch_add(1, Ordering::Relaxed);
        Self { count, previous }
    }
    pub fn rows(&self) -> usize {
        self.count.get()
    }
}
impl Drop for RowReadsForTest {
    fn drop(&mut self) {
        ACTIVE.set(self.previous.take());
        ARMED_RECORDERS.fetch_sub(1, Ordering::Relaxed);
    }
}
pub(super) fn record() {
    if ARMED_RECORDERS.load(Ordering::Relaxed) == 0 {
        return;
    }
    ACTIVE.with(|active| {
        if let Some(count) = active.borrow().as_ref() {
            count.set(count.get() + 1);
        }
    });
}
