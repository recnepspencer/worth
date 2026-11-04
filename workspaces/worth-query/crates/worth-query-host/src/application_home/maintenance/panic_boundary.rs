//! Contain scoped-worker setup, join and panic-payload disposal at the Host.
use std::panic::{catch_unwind, AssertUnwindSafe};

pub(super) fn contain(operation: impl FnOnce()) {
    // Match Execution's existing panic-boundary disposal policy. The worker's
    // domain operation itself runs inside Execution's isolated root boundary.
    if let Err(payload) = catch_unwind(AssertUnwindSafe(operation))
        && let Err(second) = catch_unwind(AssertUnwindSafe(|| drop(payload)))
    {
        // A recursively panicking destructor cannot be safely invoked again.
        std::mem::forget(second);
    }
}
