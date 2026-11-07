//! Passive observation of explicit whole-dictionary snapshot materialization.
use std::cell::Cell;
thread_local! { static ROWS: Cell<Option<u64>> = const { Cell::new(None) }; }

struct SnapshotRecording(Option<u64>, std::marker::PhantomData<std::rc::Rc<()>>);
impl Drop for SnapshotRecording {
    fn drop(&mut self) {
        ROWS.with(|rows| rows.set(self.0));
    }
}

/// Reports actual symbol rows materialized by snapshots during this thread's
/// lexical operation. No dictionary, request admission or result is changed.
pub fn observe_symbol_snapshot_materialization<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    let guard = SnapshotRecording(
        ROWS.with(|rows| rows.replace(Some(0))),
        std::marker::PhantomData,
    );
    let result = operation();
    let rows = ROWS.with(|rows| rows.get().expect("live symbol snapshot recording"));
    drop(guard);
    (result, rows)
}

pub(super) fn materialized(count: usize) {
    ROWS.with(|rows| {
        if let Some(previous) = rows.get() {
            rows.set(Some(previous.saturating_add(count as u64)));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_snapshot_recording_restores_after_unwinding() {
        let mut symbols = super::super::StringInterner::default();
        symbols.intern("retained").unwrap();
        let (snapshots, rows) = observe_symbol_snapshot_materialization(|| {
            let first = symbols.snapshot();
            assert!(std::panic::catch_unwind(|| {
                observe_symbol_snapshot_materialization(|| {
                    panic!("interrupted snapshot recording")
                });
            })
            .is_err());
            (first, symbols.snapshot())
        });
        assert_eq!(snapshots.0, snapshots.1);
        assert_eq!(
            rows,
            (snapshots.0.entries.len() + snapshots.1.entries.len()) as u64
        );
        ROWS.with(|rows| assert!(rows.get().is_none()));
    }
}
