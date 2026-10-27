use super::*;
use crate::data::resource::ResourceCompletionCommitReport;
use crate::data::temporal::TemporalWakeId;

/// A native commit remains observable even if its timeout cleanup later refuses.
pub struct SignalOwnedAsyncCompletionCommit {
    pub(super) origin: SignalOwnedAsyncSourceBinding,
    pub(super) state: SignalOwnedAsyncCompletionCommitState,
}

pub(crate) struct SignalOwnedAsyncCompletionCommitState {
    pub(crate) report: ResourceCompletionCommitReport,
    pub(crate) handle: ResourceRequestHandle,
    pub(crate) pending_timeout: Option<TemporalWakeId>,
    pub(crate) cleanup_denial: Option<SignalConditionalServiceExecutionDenial>,
}

impl SignalOwnedAsyncCompletionCommit {
    pub fn report(&self) -> &ResourceCompletionCommitReport {
        &self.state.report
    }

    pub fn timeout_cleanup_denial(&self) -> Option<&SignalConditionalServiceExecutionDenial> {
        self.state.cleanup_denial.as_ref()
    }

    pub fn timeout_cleanup_complete(&self) -> bool {
        self.state.pending_timeout.is_none()
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
        admission: &SignalOwnedAsyncCompletionAdmission,
    ) -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> {
        let binding = &admission.origin;
        self.require_owned_async_binding(binding)?;
        let admitted = admission.report.admitted_completion().ok_or_else(|| {
            SignalConditionalServiceExecutionDenial::SlotAdmission(
                crate::data::error::SignalError::invalid_input("completion admission was denied"),
            )
        })?;
        let state = self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.commit_conditional_owned_async_completion(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                admitted,
            )
        })?;
        Ok(SignalOwnedAsyncCompletionCommit {
            origin: binding.clone(),
            state,
        })
    }

    pub fn finish_owned_async_completion_timeout_cleanup(
        &self,
        committed: &mut SignalOwnedAsyncCompletionCommit,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let binding = &committed.origin;
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.finish_conditional_owned_async_completion_timeout_cleanup(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                &mut committed.state,
            )
        })
    }
}
