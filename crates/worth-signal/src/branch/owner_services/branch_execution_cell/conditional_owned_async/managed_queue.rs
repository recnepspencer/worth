use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedAdmission, SignalOwnedAsyncQueueDenial,
};
use crate::data::resource::{ResourceManagedQueueBinding, ResourceManagedQueueMutationReport};
impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn observe_conditional_owned_async_safe_point(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
    ) -> Result<
        Result<
            crate::data::resource::ResourceSafePointObservationReport,
            SignalOwnedAsyncQueueDenial,
        >,
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.observe_owned_async_safe_point(node, queue)
        })
    }
    pub(in crate::branch::owner_services) fn admit_conditional_managed_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmission, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_managed_owned_async_request(node, capacity)
        })
    }
    pub(in crate::branch::owner_services) fn mutate_conditional_owned_async_queue(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        width: u64,
        enqueue: bool,
    ) -> Result<
        Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial>,
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.mutate_owned_async_queue(node, queue, width, enqueue)
        })
    }
}
