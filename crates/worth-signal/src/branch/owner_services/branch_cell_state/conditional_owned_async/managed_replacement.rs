use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedWakeCleanup, SignalOwnedAsyncQueueDenial,
};
use crate::data::resource::{ResourceManagedQueueBinding, ResourceRequestHandle};

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn advance_managed_owned_async_timeout(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.state
            .advance_owned_async_timeout_handoff(node, handle, coordinate)
    }

    pub(in crate::branch::owner_services) fn schedule_managed_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.state.schedule_owned_async_retry_handoff(node, handle)
    }

    pub(in crate::branch::owner_services) fn advance_managed_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncQueueDenial> {
        self.state
            .advance_managed_owned_async_retry(node, queue, schedule, coordinate)
    }

    pub(in crate::branch::owner_services) fn revalidate_managed_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncQueueDenial> {
        self.state
            .revalidate_managed_owned_async_request(node, queue, intent)
    }

    pub(in crate::branch::owner_services) fn finish_managed_owned_async_wake_cleanup(
        &mut self,
        node: ResourceNodeId,
        cleanup: &mut SignalOwnedAsyncManagedWakeCleanup,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.state
            .finish_managed_owned_async_wake_cleanup(node, cleanup)
    }
}
