mod clock;
mod completion_commit;
mod initial_request;
mod managed_queue;
mod managed_replacement;
mod process_local_release;
mod wake_observation_for_test;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalOwnedAsyncRetrySchedule,
};
use crate::data::resource::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCompletionAdmissionReport, ResourceNodeId, ResourceRevalidationIntent,
};

use super::SignalBranchCellState;

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_owned_async_source(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.state.admit_owned_async_source(node)
    }

    pub(in crate::branch::owner_services) fn admit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.state.admit_owned_async_completion(node, raw)
    }

    pub(in crate::branch::owner_services) fn cancel_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.state.cancel_owned_async_request(node, handle, reason)
    }

    pub(in crate::branch::owner_services) fn owned_async_active_request_count(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.state.owned_async_active_request_count(node)
    }
}
