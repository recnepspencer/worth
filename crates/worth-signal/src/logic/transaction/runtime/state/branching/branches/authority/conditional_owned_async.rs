use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalOwnedAsyncRequestAdmission,
};
use crate::data::resource::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCompletionAdmissionReport, ResourceNodeId, ResourceRevalidationIntent,
};

use super::BranchState;

mod lifecycle;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn admit_owned_async_source(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        if !self.graph().is_alive(node.node())
            || self.derived.resource.descriptor_for_node(node).is_none()
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        Ok(())
    }

    pub(crate) fn admit_owned_async_request(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncRequestAdmission, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_request_with_lifecycle(node)
    }

    pub(crate) fn admit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_completion_with_lifecycle(node, raw)
    }

    pub(crate) fn revalidate_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        intent: ResourceRevalidationIntent,
    ) -> Result<
        crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRevalidationAdmission,
        SignalConditionalServiceExecutionDenial,
    > {
        self.revalidate_owned_async_request_with_lifecycle(node, intent)
    }

    pub(crate) fn cancel_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        handle: crate::data::resource::ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.cancel_owned_async_request_with_lifecycle(node, handle, reason)
    }
}
