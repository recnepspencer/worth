/// One compiled `catch_unwind` for owner retention calls.
///
/// Each closure type would otherwise copy the unwind boundary. Callers keep
/// the typed result in their own slot.
#[inline(never)]
pub(super) fn catch_owner_operation(
    operation: &mut dyn FnMut(),
) -> Result<(), Box<dyn std::any::Any + Send>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
}
