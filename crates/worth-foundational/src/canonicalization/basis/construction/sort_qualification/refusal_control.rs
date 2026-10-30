//! A closed certification callback can refuse qualification, never grant it.
use std::{cell::Cell, marker::PhantomData, rc::Rc};

thread_local! {
    static REFUSED: Cell<bool> = const { Cell::new(false) };
}

/// Runs this thread's callback with admitted sort qualification refused.
/// Actual compiler/target context is unchanged. Nested calls and unwinding
/// restore the previous refusal; other threads keep their real qualification.
pub fn with_refused_canonical_sort_qualification_for_test<R>(callback: impl FnOnce() -> R) -> R {
    let previous = REFUSED.replace(true);
    let _restore = RestoreRefusal {
        previous,
        _thread: PhantomData,
    };
    callback()
}

pub(super) fn qualification_refused() -> bool {
    REFUSED.get()
}

struct RestoreRefusal {
    previous: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for RestoreRefusal {
    fn drop(&mut self) {
        REFUSED.set(self.previous);
    }
}
