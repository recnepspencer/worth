//! Passive observation of payload-measurement setup in the owning scope engine.
use std::cell::Cell;
thread_local! {
    static MEASUREMENTS: Cell<Option<usize>> = const { Cell::new(None) };
}
pub(super) fn measured() {
    MEASUREMENTS.update(|count| count.map(|count| count + 1));
}
pub(crate) struct Observation;
impl Observation {
    pub(crate) fn start() -> Self {
        MEASUREMENTS.with(|count| assert_eq!(count.replace(Some(0)), None));
        Self
    }
    pub(crate) fn count(&self) -> usize {
        MEASUREMENTS.with(|count| count.get().unwrap())
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        MEASUREMENTS.set(None);
    }
}
