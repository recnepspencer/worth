//! Passive, unwind-safe recorder for the read-admission projection regression.
use super::RelationalSchemaRegistry;
use std::cell::Cell;

thread_local! {
    static PROJECTIONS: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
}

impl RelationalSchemaRegistry {
    /// Count full projections on this thread during a synchronous observation.
    /// Does not change schema validation or provide schema/runtime authority.
    pub fn observe_authority_projection_work<R>(observe: impl FnOnce() -> R) -> (R, u64) {
        PROJECTIONS.with(|count| {
            assert!(count.get().is_none(), "nested recorder");
            count.set(Some((0, 0)));
        });
        let guard = Recording;
        let result = observe();
        let work = PROJECTIONS.with(|count| count.get().expect("active recorder").1);
        drop(guard);
        (result, work)
    }
    pub fn count_authority_projections_for_test<R>(observe: impl FnOnce() -> R) -> (R, u64) {
        PROJECTIONS.with(|count| {
            assert!(count.get().is_none(), "nested recorder");
            count.set(Some((0, 0)));
        });
        let guard = Recording;
        let result = observe();
        let count = PROJECTIONS.with(|count| count.get().expect("active recorder"));
        drop(guard);
        (result, count.0)
    }
}

pub(super) fn record(rows: usize) {
    PROJECTIONS.with(|count| {
        if let Some(value) = count.get() {
            count.set(Some((value.0 + 1, value.1 + rows as u64)));
        }
    });
}

struct Recording;

impl Drop for Recording {
    fn drop(&mut self) {
        PROJECTIONS.with(|count| count.set(None));
    }
}
