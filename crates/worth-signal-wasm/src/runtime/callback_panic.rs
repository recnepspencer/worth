//! One compiled panic boundary for native callback trampolines.
use std::panic::{catch_unwind, AssertUnwindSafe};

/// One compiled `catch_unwind`. Inlining it would copy the boundary once per closure.
#[inline(never)]
pub(super) fn callback_returned(operation: &mut dyn FnMut()) -> bool {
    catch_unwind(AssertUnwindSafe(operation)).is_ok()
}
