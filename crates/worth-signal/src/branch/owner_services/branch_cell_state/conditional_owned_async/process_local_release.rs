use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedAdmissionState;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRetentionObligation;
use crate::data::resource::{
    DeniedResourceCompletion, InFlightResourceRequest, ResourceRequestHandle,
};

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn abort_process_local_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        scheduled: &crate::data::resource::ScheduledResourceRetry,
    ) -> Result<crate::data::temporal::RetiredTemporalWake, SignalConditionalServiceExecutionDenial>
    {
        self.state
            .abort_process_local_owned_async_retry(node, handle, scheduled)
    }

    pub(in crate::branch::owner_services) fn admit_process_local_managed_owned_async(
        &mut self,
        node: ResourceNodeId,
        capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.state
            .admit_managed_owned_async_request(node, capacity, true)
    }

    pub(in crate::branch::owner_services) fn release_process_local_owned_async(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        completion: Option<ResourceRequestHandle>,
        retention: Option<&SignalOwnedAsyncRetentionObligation>,
    ) -> Result<
        (InFlightResourceRequest, Option<InFlightResourceRequest>),
        SignalConditionalServiceExecutionDenial,
    > {
        self.state
            .release_process_local_owned_async(node, handle, completion, retention)
    }

    pub(in crate::branch::owner_services) fn release_process_local_owned_async_denial(
        &mut self,
        node: ResourceNodeId,
        denied: DeniedResourceCompletion,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.state
            .release_process_local_owned_async_denial(node, denied)
    }

    pub(in crate::branch::owner_services) fn release_process_local_owned_async_predecessor(
        &mut self,
        node: ResourceNodeId,
        current: ResourceRequestHandle,
        retention: &SignalOwnedAsyncRetentionObligation,
    ) -> Result<Option<InFlightResourceRequest>, SignalConditionalServiceExecutionDenial> {
        self.state
            .release_process_local_owned_async_predecessor(node, current, retention)
    }
}
