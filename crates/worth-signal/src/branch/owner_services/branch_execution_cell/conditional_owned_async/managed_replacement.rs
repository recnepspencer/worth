use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedWakeCleanup, SignalOwnedAsyncQueueDenial,
};
use crate::data::resource::ResourceManagedQueueBinding;

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn advance_conditional_managed_owned_async_timeout(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.advance_managed_owned_async_timeout(node, handle, coordinate)
        })
    }

    pub(in crate::branch::owner_services) fn schedule_conditional_managed_owned_async_retry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.schedule_managed_owned_async_retry(node, handle)
        })
    }

    pub(in crate::branch::owner_services) fn advance_conditional_managed_owned_async_retry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        queue: Option<&ResourceManagedQueueBinding>,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<
        Result<SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncQueueDenial>,
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            Ok(state.advance_managed_owned_async_retry(node, handle, queue, schedule, coordinate))
        })
    }

    pub(in crate::branch::owner_services) fn revalidate_conditional_managed_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        queue: Option<&ResourceManagedQueueBinding>,
        intent: ResourceRevalidationIntent,
        process_local: bool,
    ) -> Result<
        Result<SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncQueueDenial>,
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            Ok(state.revalidate_managed_owned_async_request(
                node,
                handle,
                queue,
                intent,
                process_local,
            ))
        })
    }

    pub(in crate::branch::owner_services) fn finish_conditional_managed_owned_async_wake_cleanup(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        cleanup: &mut SignalOwnedAsyncManagedWakeCleanup,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.finish_managed_owned_async_wake_cleanup(node, cleanup)
        })
    }
}
