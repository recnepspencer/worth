use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};

/// Contain the operation's panic and disposal of its caught payload. The boundary
/// itself is compiled once; only moving `R` out of the operation is generic.
pub(super) fn contain<R>(operation: impl FnOnce() -> R + UnwindSafe) -> Result<R, ()> {
    let mut operation = Some(operation);
    let mut outcome = None;
    contain_erased(&mut || {
        outcome = operation.take().map(|operation| operation());
    });
    outcome.ok_or(())
}

/// A payload destructor can panic too; its second payload must remain undropped.
fn contain_erased(operation: &mut dyn FnMut()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(operation))
        && let Err(second) = catch_unwind(AssertUnwindSafe(|| drop(payload)))
    {
        std::mem::forget(second);
    }
}
