//! Passive observation of projected touch work on one thread, with unwind cleanup.
use std::cell::Cell;
thread_local! { static EXAMINED: Cell<Option<u64>> = const { Cell::new(None) }; }
pub(super) fn observe<R>(observe: impl FnOnce() -> R) -> (R, u64) {
    EXAMINED.with(|examined| {
        assert!(examined.get().is_none(), "nested touch projection recorder");
        examined.set(Some(0));
    });
    let guard = Recording;
    let result = observe();
    let work = EXAMINED.with(|examined| examined.get().expect("active recorder"));
    drop(guard);
    (result, work)
}
pub(super) fn record(intents: usize) {
    EXAMINED.with(|examined| {
        if let Some(sum) = examined.get() {
            examined.set(Some(sum.saturating_add(intents as u64)));
        }
    });
}
struct Recording;
impl Drop for Recording {
    fn drop(&mut self) {
        EXAMINED.with(|examined| examined.set(None));
    }
}
