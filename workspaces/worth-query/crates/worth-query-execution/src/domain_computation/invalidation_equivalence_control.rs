//! Thread-bound differential verification, armed only by its owned guard.
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
thread_local! { static LISTENERS: Cell<usize> = const { Cell::new(0) }; }

/// Shadows optimized invalidation with the full-image differential oracle.
/// Dropping the final guard restores ordinary verification on this thread.
#[must_use = "the differential oracle is armed only while the guard is held"]
pub struct WorthQueryInvalidationEquivalence {
    _thread: PhantomData<Rc<()>>,
}
impl WorthQueryInvalidationEquivalence {
    /// Arm full-image verification until this thread drops its final guard.
    pub fn arm_on_this_thread() -> Self {
        LISTENERS.with(|count| {
            count.set(
                count
                    .get()
                    .checked_add(1)
                    .expect("oracle guard count overflow"),
            )
        });
        Self {
            _thread: PhantomData,
        }
    }
}
impl Drop for WorthQueryInvalidationEquivalence {
    fn drop(&mut self) {
        LISTENERS.with(|count| count.set(count.get() - 1));
    }
}
pub(in crate::domain_computation) fn is_armed() -> bool {
    LISTENERS.with(Cell::get) != 0
}
