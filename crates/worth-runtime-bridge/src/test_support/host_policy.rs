//! Test host construction, unavailable on Bridge's production surface.
use crate::policy::BridgeExecutionPolicyBaseline;

impl BridgeExecutionPolicyBaseline {
    pub(crate) fn serial_request(
        self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
    ) -> worth_execution::SerialRequest {
        worth_execution::SerialRequest::from_policy(&self.request_policy(), cancellation, deadline)
    }
}
