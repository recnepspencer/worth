use std::panic::{catch_unwind, UnwindSafe};

/// Contain the operation's panic and disposal of its caught payload. A payload
/// destructor can panic too; its second payload must then remain undropped.
pub(super) fn contain<R>(operation: impl FnOnce() -> R + UnwindSafe) -> Result<R, ()> {
    catch_unwind(operation).map_err(|payload| {
        if let Err(second) = catch_unwind(std::panic::AssertUnwindSafe(|| drop(payload))) {
            std::mem::forget(second);
        }
    })
}
