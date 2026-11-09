//! Passive observation of the staged-intent replay regression, with unwind cleanup.
use std::cell::Cell;
thread_local! { static VISITS: Cell<Option<u64>> = const { Cell::new(None) }; }
impl super::super::BranchBoundRelationalTransaction {
    pub fn observe_read_projection_work<R>(observe: impl FnOnce() -> R) -> (R, u64) {
        VISITS.with(|v| {
            assert!(v.get().is_none(), "nested projection recorder");
            v.set(Some(0));
        });
        let guard = Recording;
        let result = observe();
        let work = VISITS.with(|v| v.get().expect("active recorder"));
        drop(guard);
        (result, work)
    }
}
pub(super) fn record() {
    VISITS.with(|v| {
        if let Some(n) = v.get() {
            v.set(Some(n + 1));
        }
    });
}
struct Recording;
impl Drop for Recording {
    fn drop(&mut self) {
        VISITS.with(|v| v.set(None));
    }
}
