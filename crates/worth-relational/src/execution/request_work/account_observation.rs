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
pub(crate) fn observe<R>(body: impl FnOnce() -> R) -> (R, Vec<u64>) {
    let previous = CHARGES.with(|slot| slot.replace(Some(Vec::new())));
    let _restore = Restore(previous);
    let outcome = body();
    let charges = CHARGES.with(|slot| slot.borrow_mut().take().unwrap());
    (outcome, charges)
}
pub(super) fn record(work: u64) {
    CHARGES.with(|slot| {
        if let Some(charges) = slot.borrow_mut().as_mut() {
            charges.push(work);
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn original_work_recording_restores_its_previous_observer_after_unwinding() {
        use super::{observe, record};
        let (_, outer) = observe(|| {
            record(3);
            let refusal = std::panic::catch_unwind(|| {
                observe(|| {
                    record(7);
                    panic!("original observer unwind");
                })
            });
            assert!(refusal.is_err());
            record(5);
        });
        assert_eq!(outer, vec![3, 5]);
        let (_, empty) = observe(|| ());
        assert!(empty.is_empty());
    }
}
