//! Bridge tests declare policy at their test host entry.
use crate::policy::BridgeExecutionPolicyBaseline;
use worth_execution::{CancellationToken, ExecutionRequest};
#[path = "test_support/host_policy.rs"]
mod host_policy;

pub(crate) fn with_declared_request<R>(
    policy: BridgeExecutionPolicyBaseline,
    execute: impl for<'request> FnOnce(ExecutionRequest<'request, 'request>) -> R,
) -> R {
    let serial = policy.serial_request(CancellationToken::new(), None);
    execute(ExecutionRequest::serial(&serial))
}
