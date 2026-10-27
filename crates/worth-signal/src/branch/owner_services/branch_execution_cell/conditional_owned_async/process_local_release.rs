use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedAdmissionState;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRetentionObligation;
use crate::data::resource::{DeniedResourceCompletion, InFlightResourceRequest};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn abort_conditional_process_local_owned_async_retry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        scheduled: &crate::data::resource::ScheduledResourceRetry,
    ) -> Result<crate::data::temporal::RetiredTemporalWake, SignalConditionalServiceExecutionDenial>
    {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.abort_process_local_owned_async_retry(node, handle, scheduled)
        })
    }

    pub(in crate::branch::owner_services) fn admit_conditional_process_local_managed_owned_async(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_process_local_managed_owned_async(node, capacity)
        })
    }

    pub(in crate::branch::owner_services) fn release_conditional_process_local_owned_async(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        completion: Option<ResourceRequestHandle>,
        retention: Option<&SignalOwnedAsyncRetentionObligation>,
    ) -> Result<
        (InFlightResourceRequest, Option<InFlightResourceRequest>),
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.release_process_local_owned_async(node, handle, completion, retention)
        })
    }

    pub(in crate::branch::owner_services) fn release_conditional_process_local_owned_async_denial(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        denied: DeniedResourceCompletion,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.release_process_local_owned_async_denial(node, denied)
        })
    }

    pub(in crate::branch::owner_services) fn release_conditional_process_local_predecessor(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        current: ResourceRequestHandle,
        retention: &SignalOwnedAsyncRetentionObligation,
    ) -> Result<Option<InFlightResourceRequest>, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.release_process_local_owned_async_predecessor(node, current, retention)
        })
    }
}
