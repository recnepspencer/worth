use std::panic::{catch_unwind, AssertUnwindSafe};

/// One compiled `catch_unwind` for every durability worker.
///
/// The worker bodies differ. The unwind boundary does not, and inlining it
/// would copy that boundary once per closure.
#[inline(never)]
pub(super) fn contain(operation: &mut dyn FnMut()) -> Result<(), ()> {
    catch_unwind(AssertUnwindSafe(operation))
        .map(|_| ())
        .map_err(|_| ())
}
