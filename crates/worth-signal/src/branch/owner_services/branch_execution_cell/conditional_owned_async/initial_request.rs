use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedAdmissionState;

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_conditional_unqueued_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_unqueued_owned_async_request(node)
        })
    }
}
