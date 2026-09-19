use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalOwnedAsyncRequestAdmission,
    SignalOwnedAsyncRetryAdmission, SignalOwnedAsyncRetrySchedule,
    SignalOwnedAsyncRevalidationAdmission, SignalOwnedAsyncTimeoutAdmission,
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

    pub(in crate::branch::owner_services) fn admit_owned_async_request(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncRequestAdmission, SignalConditionalServiceExecutionDenial> {
        self.state.admit_owned_async_request(node)
    }

    pub(in crate::branch::owner_services) fn admit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.state.admit_owned_async_completion(node, raw)
    }

    pub(in crate::branch::owner_services) fn revalidate_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncRevalidationAdmission, SignalConditionalServiceExecutionDenial>
    {
        self.state.revalidate_owned_async_request(node, intent)
    }

    pub(in crate::branch::owner_services) fn cancel_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.state.cancel_owned_async_request(node, handle, reason)
    }

    pub(in crate::branch::owner_services) fn advance_owned_async_request_to_timeout(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncTimeoutAdmission, SignalConditionalServiceExecutionDenial> {
        self.state
            .advance_owned_async_request_to_timeout(node, handle, coordinate)
    }

    pub(in crate::branch::owner_services) fn schedule_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncRetrySchedule, SignalConditionalServiceExecutionDenial> {
        self.state.schedule_owned_async_retry(node, handle)
    }

    pub(in crate::branch::owner_services) fn advance_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncRetryAdmission, SignalConditionalServiceExecutionDenial> {
        self.state
            .advance_owned_async_retry(node, handle, schedule, coordinate)
    }

    pub(in crate::branch::owner_services) fn owned_async_active_request_count(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.state.owned_async_active_request_count(node)
    }
}
