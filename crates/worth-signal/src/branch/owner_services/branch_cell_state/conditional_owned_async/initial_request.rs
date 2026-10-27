use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedAdmissionState;

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_unqueued_owned_async_request(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.state.admit_owned_async_request_handoff(node, true)
    }
}
