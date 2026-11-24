//! Observe the maintenance memory reservation in its existing request scope.
use std::cell::Cell;

thread_local! {
    static OBSERVED: Cell<Option<u64>> = const { Cell::new(None) };
}

pub(super) fn record(bytes: u64) {
    OBSERVED.with(|slot| {
        if slot.get().is_some() {
            slot.set(Some(bytes));
        }
    });
}

#[doc(hidden)]
pub fn record_allocation_for_test<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            OBSERVED.with(|slot| slot.set(None));
        }
    }

    OBSERVED.with(|slot| {
        assert!(slot.get().is_none(), "one allocation observation per scope");
        slot.set(Some(0));
    });
    let _reset = Reset;
    let result = operation();
    let bytes = OBSERVED.with(|slot| slot.get().expect("the recorder is active"));
    (result, bytes)
}
