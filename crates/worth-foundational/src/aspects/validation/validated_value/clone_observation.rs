use std::cell::Cell;
thread_local! { static CLONES: Cell<Option<usize>> = const { Cell::new(None) }; }

/// A read-only observation loan. Unwinding restores the prior observer state.
pub(in crate::aspects) struct CloneObservation(Option<usize>);
impl CloneObservation {
    pub(in crate::aspects) fn start() -> Self {
        Self(CLONES.replace(Some(0)))
    }
    pub(in crate::aspects) fn work(&self) -> usize {
        CLONES.get().unwrap_or(0)
    }
}
impl Drop for CloneObservation {
    fn drop(&mut self) {
        CLONES.set(self.0);
    }
}
pub(super) fn record_clone() {
    CLONES.set(CLONES.get().map(|count| count.saturating_add(1)));
}
