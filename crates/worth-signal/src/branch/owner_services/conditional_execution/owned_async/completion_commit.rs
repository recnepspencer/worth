use super::*;
use crate::data::resource::{AdmittedResourceCompletion, ResourceCompletionCommitReport};
use crate::data::temporal::TemporalWakeId;

/// A native commit remains observable even if its timeout cleanup later refuses.
pub struct SignalOwnedAsyncCompletionCommit {
    pub(crate) report: ResourceCompletionCommitReport,
    pub(crate) handle: ResourceRequestHandle,
    pub(crate) pending_timeout: Option<TemporalWakeId>,
    pub(crate) cleanup_denial: Option<SignalConditionalServiceExecutionDenial>,
}

impl SignalOwnedAsyncCompletionCommit {
    pub fn report(&self) -> &ResourceCompletionCommitReport {
        &self.report
    }

    pub fn timeout_cleanup_denial(&self) -> Option<&SignalConditionalServiceExecutionDenial> {
        self.cleanup_denial.as_ref()
    }

    pub fn timeout_cleanup_complete(&self) -> bool {
        self.pending_timeout.is_none()
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn commit_owned_async_completion(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        admitted: AdmittedResourceCompletion,
    ) -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.commit_conditional_owned_async_completion(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                admitted,
            )
        })
    }

    pub fn finish_owned_async_completion_timeout_cleanup(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        committed: &mut SignalOwnedAsyncCompletionCommit,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.finish_conditional_owned_async_completion_timeout_cleanup(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                committed,
            )
        })
    }
}
