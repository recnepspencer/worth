/// One compiled `catch_unwind` for every caller.
///
/// `catch_unwind` is generic over the closure. Compiling it at each call site
/// copies the unwind boundary once per closure type. Callers keep any typed
/// result in their own slot and pass the operation through this object.
#[inline(never)]
pub(crate) fn catch_operation(
    operation: &mut dyn FnMut(),
) -> Result<(), Box<dyn std::any::Any + Send>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
}
