//! Read-only original-account reports; recording neither admits nor charges work.
use std::cell::RefCell;
thread_local! {
    static CHARGES: RefCell<Option<Vec<u64>>> = const { RefCell::new(None) };
}
struct Restore(Option<Vec<u64>>);
impl Drop for Restore {
    fn drop(&mut self) {
        CHARGES.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
/// Scoped observer of actual original-account settlements on the current thread.
/// It grants no request, budget or publication authority.
pub struct RelationalWorkAccountRecorderForTest;
impl RelationalWorkAccountRecorderForTest {
    pub const fn new() -> Self {
        Self
    }
    pub fn record_for_test<R>(&self, body: impl FnOnce() -> R) -> (R, Vec<u64>) {
        let previous = CHARGES.with(|slot| slot.replace(Some(Vec::new())));
        let _restore = Restore(previous);
        let outcome = body();
        let charges = CHARGES.with(|slot| slot.borrow_mut().take().unwrap());
        (outcome, charges)
    }
}
pub(super) fn record(work: u64) {
    CHARGES.with(|slot| {
        if let Some(charges) = slot.borrow_mut().as_mut() {
            charges.push(work);
        }
    });
}
