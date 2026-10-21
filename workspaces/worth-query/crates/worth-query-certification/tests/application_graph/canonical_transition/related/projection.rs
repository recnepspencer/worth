//! Callback observation and cancellation remain owned by the actual projection.
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
thread_local! {
 static COUNT:std::cell::Cell<u32>=const {std::cell::Cell::new(0)};
 static CANCEL:std::cell::RefCell<Option<WorthQueryCancellationSource>>=const {std::cell::RefCell::new(None)};
 static REJECT:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};
}
pub fn reset() {
    COUNT.set(0);
    CANCEL.with_borrow_mut(|source| *source = None);
    REJECT.set(false);
}
pub fn count() -> u32 {
    COUNT.get()
}
pub fn returned() -> bool {
    COUNT.set(COUNT.get() + 1);
    CANCEL.with_borrow_mut(|source| {
        if let Some(source) = source.take() {
            source.cancel();
        }
    });
    REJECT.get()
}
pub struct Guard;
impl Guard {
    pub fn arm(source: WorthQueryCancellationSource, reject: bool) -> Self {
        reset();
        CANCEL.with_borrow_mut(|slot| *slot = Some(source));
        REJECT.set(reject);
        Self
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        reset();
    }
}
