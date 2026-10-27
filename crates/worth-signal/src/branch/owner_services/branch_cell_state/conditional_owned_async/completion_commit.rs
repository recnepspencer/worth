use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncCompletionCommit;
use crate::data::resource::AdmittedResourceCompletion;
impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn commit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        admitted: AdmittedResourceCompletion,
    ) -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> {
        self.state.commit_owned_async_completion(node, admitted)
    }
    pub(in crate::branch::owner_services) fn finish_owned_async_completion_timeout_cleanup(
        &mut self,
        node: ResourceNodeId,
        committed: &mut SignalOwnedAsyncCompletionCommit,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.state
            .finish_owned_async_completion_timeout_cleanup(node, committed)
    }
}
