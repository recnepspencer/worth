//! Owned observation lifetime for Query execution, independent of request authority.
use std::cell::{Cell, RefCell};
pub mod owned_control_scope;
thread_local! {
    static LISTENERS: Cell<usize> = const { Cell::new(0) };
    static CLEANUPS: RefCell<Vec<Box<dyn FnOnce()>>> = const { RefCell::new(Vec::new()) };
    static ADVANCEMENT_ACTIVE_PROBE: Cell<Option<fn() -> bool>> = const { Cell::new(None) };
}

/// Lower owned guard that tracks active thread-local listeners and runs cleanups on final drop.
#[must_use = "the observation ends when its guard is dropped"]
pub struct ExecutionObservationGuard {
    _controls: owned_control_scope::OwnedControlScope,
    _signal: worth_signal::facade::SignalRequestWorkObservation,
    _world: worth_runtime_world::facade::RuntimeWorldReadObservation,
}

impl ExecutionObservationGuard {
    pub fn install_on_this_thread() -> Self {
        let count = LISTENERS
            .with(Cell::get)
            .checked_add(1)
            .expect("Query listener count overflow");
        let observation = Self {
            _controls: owned_control_scope::OwnedControlScope::install(),
            _signal: worth_signal::facade::SignalRequestWorkObservation::install_on_this_thread(),
            _world:
                worth_runtime_world::facade::RuntimeWorldReadObservation::install_on_this_thread(),
        };
        LISTENERS.with(|listeners| listeners.set(count));
        observation
    }

    pub fn advancement_is_active(&self) -> bool {
        ADVANCEMENT_ACTIVE_PROBE
            .with(Cell::get)
            .is_some_and(|probe| probe())
    }
}

impl Drop for ExecutionObservationGuard {
    fn drop(&mut self) {
        let remaining = LISTENERS.with(|listeners| {
            let remaining = listeners.get() - 1;
            listeners.set(remaining);
            remaining
        });
        if remaining == 0 {
            ADVANCEMENT_ACTIVE_PROBE.with(|probe| probe.set(None));
            let cleanups = CLEANUPS.with(|cleanups| std::mem::take(&mut *cleanups.borrow_mut()));
            for cleanup in cleanups {
                cleanup();
            }
        }
    }
}

pub fn is_installed() -> bool {
    LISTENERS.with(Cell::get) != 0
}

pub fn register_cleanup(cleanup: fn()) {
    CLEANUPS.with(|cleanups| cleanups.borrow_mut().push(Box::new(cleanup)));
}

pub fn register_advancement_active_probe(probe: fn() -> bool) {
    ADVANCEMENT_ACTIVE_PROBE.with(|slot| slot.set(Some(probe)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_owned_listeners_end_only_with_the_final_guard() {
        assert!(!is_installed());
        let outer = ExecutionObservationGuard::install_on_this_thread();
        assert!(is_installed());
        let inner = ExecutionObservationGuard::install_on_this_thread();
        drop(outer);
        assert!(is_installed());
        drop(inner);
        assert!(!is_installed());
        CLEANUPS.with(|cleanups| assert!(cleanups.borrow().is_empty()));
    }
}
