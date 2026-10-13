//! Request refusals retain their owner when a scope transports a domain-shaped cause.
use super::{
    scope::{run_scope_with_charge, ScopeOutcome},
    KernelContext, KernelFailure, KernelStop,
};
use crate::{authority::ExecutionResourceLease, ChargedBytes};

pub(crate) enum ScopeKernelFailure<E> {
    /// The producing owner refused a request resource or its accounting.
    Refused(E),
    Kernel(KernelFailure<E>),
}
impl<E> From<KernelStop> for ScopeKernelFailure<E> {
    fn from(stop: KernelStop) -> Self {
        Self::Kernel(KernelFailure::Stop(stop))
    }
}
/// One adapter to the existing scope; admission, metering and settlement are shared.
pub(crate) fn run_admitting_scope<R: ChargedBytes, E: ChargedBytes>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    charge_parent: bool,
    kernel: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<R, ScopeKernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    run_scope_with_charge(
        lease,
        retained_bytes,
        max_result_bytes,
        charge_parent,
        |context| {
            kernel(context).map_err(|failure| match failure {
                ScopeKernelFailure::Refused(cause) => {
                    KernelFailure::Domain(context.refuse_request(cause))
                }
                ScopeKernelFailure::Kernel(failure) => failure,
            })
        },
    )
}
