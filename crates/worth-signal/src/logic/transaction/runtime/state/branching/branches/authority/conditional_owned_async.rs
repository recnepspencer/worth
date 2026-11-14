use crate::branch::owner_services::conditional_execution::SignalConditionalServiceExecutionDenial;
use crate::data::resource::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCompletionAdmissionReport, ResourceNodeId,
};

use super::BranchState;

mod lifecycle;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn readmit_owned_async_definition(
        &mut self,
        declaration: crate::data::resource::ResourceNodeDeclaration,
    ) -> Result<
        crate::data::resource::LoweredResourceDescriptor,
        SignalConditionalServiceExecutionDenial,
    > {
        use SignalConditionalServiceExecutionDenial as D;
        let node = declaration.node();
        if !self.graph().is_alive(node.node()) {
            return Err(D::DefinitionMismatch);
        }
        if let Some(existing) = self.derived.resource.descriptor_for_node(node) {
            let payload = declaration.payload_contract();
            if existing.payload_contract_digest()
                != &crate::data::resource::ResourcePayloadContractDigest::from_contract(
                    payload.id(),
                    payload.max_payload_bytes(),
                )
                || !self
                    .derived
                    .resource
                    .classify_policy_compatibility_optional(&declaration, None)
                    .map_err(D::SlotAdmission)?
                    .is_compatible()
            {
                return Err(D::DefinitionMismatch);
            }
            return Ok(existing.clone());
        }
        self.derived
            .resource
            .declare_resource_node(declaration, None)
            .map_err(D::SlotAdmission)?;
        self.derived
            .resource
            .descriptor_for_node(node)
            .cloned()
            .ok_or(D::DefinitionMismatch)
    }

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

    pub(crate) fn admit_owned_async_completion(
        &mut self,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_completion_with_lifecycle(node, raw)
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
