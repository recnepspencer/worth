//! Owned observation lifetime for Query execution, independent of request authority.
use super::execution_runtime::source_invalidation::NativeReservationObservation;
use std::cell::{Cell, RefCell};
thread_local! {
    static LISTENERS: Cell<usize> = const { Cell::new(0) };
    static CLEANUPS: RefCell<Vec<fn()>> = const { RefCell::new(Vec::new()) };
}

/// Records this thread's Query work while an owned listener exists.
/// It grants no execution, publication or installation authority.
#[must_use = "the observation ends when its guard is dropped"]
pub struct WorthQueryExecutionObservation {
    _signal: worth_signal::facade::SignalRequestWorkObservation,
    _world: worth_runtime_world::facade::RuntimeWorldReadObservation,
    _native: NativeReservationObservation,
}
impl WorthQueryExecutionObservation {
    /// Install passive Query, Signal and World observation on this thread.
    /// Take retained report vectors before dropping the final listener.
    pub fn install_on_this_thread() -> Self {
        let count = LISTENERS
            .with(Cell::get)
            .checked_add(1)
            .expect("Query listener count overflow");
        let observation = Self {
            _signal: worth_signal::facade::SignalRequestWorkObservation::install_on_this_thread(),
            _world:
                worth_runtime_world::facade::RuntimeWorldReadObservation::install_on_this_thread(),
            _native: NativeReservationObservation::install(),
        };
        LISTENERS.with(|listeners| listeners.set(count));
        observation
    }
}
impl Drop for WorthQueryExecutionObservation {
    fn drop(&mut self) {
        let remaining = LISTENERS.with(|listeners| {
            let remaining = listeners.get() - 1;
            listeners.set(remaining);
            remaining
        });
        if remaining == 0 {
            let cleanups = CLEANUPS.with(|cleanups| std::mem::take(&mut *cleanups.borrow_mut()));
            for cleanup in cleanups {
                cleanup();
            }
        }
    }
}
pub(in crate::domain_computation) fn is_installed() -> bool {
    LISTENERS.with(Cell::get) != 0
}
pub(in crate::domain_computation) fn register_cleanup(cleanup: fn()) {
    CLEANUPS.with(|cleanups| cleanups.borrow_mut().push(cleanup));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_owned_listeners_end_only_with_the_final_guard() {
        assert!(!is_installed());
        let outer = WorthQueryExecutionObservation::install_on_this_thread();
        assert!(is_installed());
        let inner = WorthQueryExecutionObservation::install_on_this_thread();
        drop(outer);
        assert!(is_installed());
        drop(inner);
        assert!(!is_installed());
        CLEANUPS.with(|cleanups| assert!(cleanups.borrow().is_empty()));
    }
}
