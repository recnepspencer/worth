use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRetentionObligation;
use crate::data::resource::{
    DeniedResourceCompletion, InFlightResourceRequest, ResourceRetentionPolicyDeclaration,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn require_process_local_owned_async(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        if self
            .derived
            .resource
            .descriptor_for_node(node)
            .is_none_or(|descriptor| {
                descriptor.retention_policy()
                    != &ResourceRetentionPolicyDeclaration::TerminalSummariesOnly
            })
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        Ok(())
    }

    pub(crate) fn release_process_local_owned_async(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        completion: Option<ResourceRequestHandle>,
        retention: Option<&SignalOwnedAsyncRetentionObligation>,
    ) -> Result<
        (InFlightResourceRequest, Option<InFlightResourceRequest>),
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_process_local_owned_async(node)?;
        self.require_owned_async_handle(node, handle)?;
        let terminal = self
            .derived
            .resource
            .prepare_process_local_release(node, handle)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        if terminal.status() == crate::data::resource::ResourceInFlightStatus::Fulfilled
            && completion != Some(handle)
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        if terminal
            .timeout_wake_id()
            .is_some_and(|wake| !self.derived.temporal.can_release_resource_wake(node, wake))
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let predecessor = retention
            .map(|value| self.prepare_process_local_predecessor(node, value))
            .transpose()?
            .flatten();
        // All fallible validation precedes either owner's mutation under this
        // same cell lock. The returned terminal is genuine release evidence.
        if let Some(retention) = retention {
            self.commit_process_local_predecessor(
                node,
                retention,
                predecessor.as_ref(),
                Some(handle),
            );
        }
        if let Some(wake) = terminal.timeout_wake_id() {
            self.derived.temporal.release_resource_wake(wake);
        }
        self.derived
            .resource
            .release_process_local_record(terminal.clone(), None);
        Ok((terminal, predecessor))
    }

    pub(crate) fn release_process_local_owned_async_denial(
        &mut self,
        node: ResourceNodeId,
        denied: DeniedResourceCompletion,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.require_process_local_owned_async(node)?;
        if !self
            .derived
            .resource
            .release_process_local_denial(node, denied)
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        Ok(())
    }

    pub(crate) fn release_process_local_owned_async_predecessor(
        &mut self,
        node: ResourceNodeId,
        current: ResourceRequestHandle,
        retention: &SignalOwnedAsyncRetentionObligation,
    ) -> Result<Option<InFlightResourceRequest>, SignalConditionalServiceExecutionDenial> {
        self.require_process_local_owned_async(node)?;
        let actual = self.require_owned_async_request_instance(node, current)?;
        if !actual.is_process_local() {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let terminal = self.prepare_process_local_predecessor(node, retention)?;
        self.commit_process_local_predecessor(node, retention, terminal.as_ref(), Some(current));
        Ok(terminal)
    }

    fn prepare_process_local_predecessor(
        &self,
        node: ResourceNodeId,
        retention: &SignalOwnedAsyncRetentionObligation,
    ) -> Result<Option<InFlightResourceRequest>, SignalConditionalServiceExecutionDenial> {
        let terminal = retention
            .previous
            .map(|handle| {
                self.derived
                    .resource
                    .prepare_process_local_release(node, handle)
                    .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)
            })
            .transpose()?;
        if terminal.as_ref().is_some_and(|value| {
            value.status() == crate::data::resource::ResourceInFlightStatus::Fulfilled
        }) || retention
            .wakes
            .iter()
            .flatten()
            .any(|wake| !self.derived.temporal.can_release_resource_wake(node, *wake))
            || retention.retry.as_ref().is_some_and(|retry| {
                !self
                    .derived
                    .resource
                    .process_local_retry_lineage_matches(node, retry)
            })
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        if let Some(terminal) = &terminal {
            if terminal
                .timeout_wake_id()
                .is_some_and(|wake| !self.derived.temporal.can_release_resource_wake(node, wake))
            {
                return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
            }
        }
        Ok(terminal)
    }

    fn commit_process_local_predecessor(
        &mut self,
        node: ResourceNodeId,
        retention: &SignalOwnedAsyncRetentionObligation,
        terminal: Option<&InFlightResourceRequest>,
        current: Option<ResourceRequestHandle>,
    ) {
        // No fallible handoff follows removal. Request-generation budget is
        // deliberately preserved: an admitted retry shares that generation.
        for wake in retention.wakes.iter().flatten() {
            self.derived.temporal.release_resource_wake(*wake);
        }
        if let Some(retry) = &retention.retry {
            self.derived
                .resource
                .release_process_local_retry_lineage(node, retry);
        }
        if let Some(terminal) = terminal {
            if let Some(wake) = terminal.timeout_wake_id() {
                self.derived.temporal.release_resource_wake(wake);
            }
            self.derived
                .resource
                .release_process_local_record(terminal.clone(), current);
        }
    }
}
