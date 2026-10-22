use super::*;

/// The same scope owner without an application payload admission contract.
/// Its only retained allocation is its real framework context; nested patterns
/// retain their own cells. Request and computation work controls remain intact.
pub(crate) fn run_scope_native_within<R, E>(
    lease: Option<&ExecutionResourceLease<'_>>,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    kernel: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    run_scope_inner(
        lease,
        0,
        None,
        charge_parent,
        work_ceiling,
        serial,
        None,
        |context, _| kernel(context),
        PayloadStorage::Native,
    )
}
