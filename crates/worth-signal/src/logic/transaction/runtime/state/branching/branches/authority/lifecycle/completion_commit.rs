use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncCompletionCommit;
use crate::data::resource::AdmittedResourceCompletion;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn commit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        admitted: AdmittedResourceCompletion,
    ) -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_handle(node, admitted.handle())?;
        let actual = self.require_in_flight(admitted.handle())?;
        if admitted.node() != node
            || actual.attempt() != admitted.attempt()
            || actual.descriptor_id() != admitted.descriptor_id()
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let timeout = self
            .derived
            .resource
            .active_timeout_wake_for_handle(admitted.handle());
        let capture = self.captures_telemetry();
        let staged = self
            .derived
            .resource
            .stage_admitted_resource_completion(
                admitted,
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        let report = self
            .derived
            .resource
            .commit_staged_resource_completion(
                staged.staged_effect(),
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        let mut committed = SignalOwnedAsyncCompletionCommit {
            report,
            handle: admitted.handle(),
            pending_timeout: timeout,
            cleanup_denial: None,
        };
        // The commit has happened. A cleanup refusal cannot erase that evidence.
        if let Err(cause) = self.finish_owned_async_completion_timeout_cleanup(node, &mut committed)
        {
            committed.cleanup_denial = Some(cause);
        }
        Ok(committed)
    }

    pub(crate) fn finish_owned_async_completion_timeout_cleanup(
        &mut self,
        node: ResourceNodeId,
        committed: &mut SignalOwnedAsyncCompletionCommit,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        if committed.report.lifecycle().node() != node {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        if committed.pending_timeout.is_none() {
            return Ok(());
        }
        self.require_owned_async_handle(node, committed.handle)?;
        self.retire_optional_wake(
            committed.pending_timeout,
            TemporalWakeRetirementReason::Consumed,
        )?;
        committed.pending_timeout = None;
        committed.cleanup_denial = None;
        Ok(())
    }
}
