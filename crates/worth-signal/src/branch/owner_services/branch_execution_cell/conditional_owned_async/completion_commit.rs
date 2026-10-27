use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncCompletionCommit;
use crate::data::resource::AdmittedResourceCompletion;
impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn commit_conditional_owned_async_completion(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        admitted: AdmittedResourceCompletion,
    ) -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.commit_owned_async_completion(node, admitted)
        })
    }
    pub(in crate::branch::owner_services) fn finish_conditional_owned_async_completion_timeout_cleanup(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        committed: &mut SignalOwnedAsyncCompletionCommit,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.finish_owned_async_completion_timeout_cleanup(node, committed)
        })
    }
}
