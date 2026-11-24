//! A native scan limit and the caller's execution stop are distinct failures.
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeScanStop<Limit> {
    Request(Cause),
    Limit(Limit),
}
impl<Limit> NativeScanStop<Limit> {
    pub fn request(stop: worth_execution::MapKernelStop) -> Self {
        Self::Request(crate::execution::denial::kernel_stop(stop))
    }
}
