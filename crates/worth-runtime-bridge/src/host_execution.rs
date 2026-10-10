//! Hosts with no Query caller declare their request policy at this entry.
use crate::policy::BridgeExecutionPolicyBaseline;
#[cfg(test)]
use worth_execution::{CancellationToken, ExecutionRequest};

#[cfg(test)]
pub(crate) fn with_declared_request<R>(
    policy: BridgeExecutionPolicyBaseline,
    execute: impl for<'request> FnOnce(ExecutionRequest<'request, 'request>) -> R,
) -> R {
    let serial = policy.serial_request(CancellationToken::new(), None);
    execute(ExecutionRequest::serial(&serial))
}

impl BridgeExecutionPolicyBaseline {
    pub fn serial_request(
        self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
    ) -> worth_execution::SerialRequest {
        worth_execution::SerialRequest::from_policy(&self.request_policy(), cancellation, deadline)
    }
}
