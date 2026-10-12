//! Unit-only recorder of native entity materialization, scoped to its owner thread.
use std::{cell::RefCell, rc::Rc};
thread_local! { static ACTIVE: RefCell<Option<Rc<std::cell::Cell<usize>>>> = const { RefCell::new(None) }; }
pub(crate) struct RowReadsForTest {
    count: Rc<std::cell::Cell<usize>>,
    previous: Option<Rc<std::cell::Cell<usize>>>,
}
impl RowReadsForTest {
    pub(crate) fn install_on_this_thread() -> Self {
        let count = Rc::new(std::cell::Cell::new(0));
        let previous = ACTIVE.replace(Some(Rc::clone(&count)));
        Self { count, previous }
    }
    pub(crate) fn rows(&self) -> usize {
        self.count.get()
    }
}
impl Drop for RowReadsForTest {
    fn drop(&mut self) {
        ACTIVE.set(self.previous.take());
    }
}
pub(super) fn record() {
    ACTIVE.with(|active| {
        if let Some(count) = active.borrow().as_ref() {
            count.set(count.get() + 1);
        }
    });
}
