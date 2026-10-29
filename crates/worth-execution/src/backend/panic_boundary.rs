use std::panic::{catch_unwind, AssertUnwindSafe};

/// Contain the operation's panic and disposal of its caught payload. A payload
/// destructor can panic too; its second payload must then remain undropped.
///
/// The operation is a trampoline. One compiled boundary serves every caller;
/// inlining it would copy `catch_unwind` once per closure type. The assertion
/// is the same unwind-safety claim each caller used to make at the call site.
/// A typed result stays in the caller's slot; this boundary returns only
/// whether the operation panicked.
#[inline(never)]
pub(crate) fn contain(operation: &mut dyn FnMut()) -> Result<(), ()> {
    catch_unwind(AssertUnwindSafe(operation)).map_err(|payload| {
        if let Err(second) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            std::mem::forget(second);
        }
    })
}
