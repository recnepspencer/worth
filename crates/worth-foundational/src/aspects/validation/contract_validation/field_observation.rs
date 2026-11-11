//! Owner-local observation of actual field validation, with unwind-safe nesting.
use std::cell::Cell;
thread_local! { static VISITS: Cell<Option<usize>> = const { Cell::new(None) }; }
pub(in crate::aspects::validation::contract_validation) fn record() {
    VISITS.with(|visits| {
        if let Some(count) = visits.get() {
            visits.set(Some(count + 1));
        }
    });
}
pub(in crate::aspects) struct FieldValidationObservation(Option<usize>);
impl FieldValidationObservation {
    pub(in crate::aspects) fn start() -> Self {
        Self(VISITS.replace(Some(0)))
    }
    pub(in crate::aspects) fn work(&self) -> usize {
        VISITS.get().unwrap()
    }
}
impl Drop for FieldValidationObservation {
    fn drop(&mut self) {
        VISITS.set(self.0);
    }
}
