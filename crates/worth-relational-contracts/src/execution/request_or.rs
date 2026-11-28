//! Relay a request refusal or the operation's own typed refusal.
use super::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestOr<E> {
    Request(RelationalExecutionDenialCause),
    Inner(E),
}

impl<E> RequestOr<E> {
    pub fn from_kernel_stop(stop: worth_execution::MapKernelStop) -> Self {
        Self::Request(super::denial::kernel_stop(stop))
    }
}

/// Relay an SDK execution stop or the operation's own typed refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionOr<E> {
    Request(worth_execution::WorkCeilingDenial),
    Inner(E),
}

impl<E> From<E> for ExecutionOr<E> {
    fn from(cause: E) -> Self {
        Self::Inner(cause)
    }
}
