//! Sealed test doors over this module's private members.
use super::*;

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn elevation_for_test(&self) -> &Option<worth_relational::facade::identity::EntityId> {
        &self.elevation
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn data_mut_for_test(&mut self) -> &mut Arc<RetainedCapabilityRequestData> {
        &mut self.data
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn matches_elevated_request_for_test(
        &self,
        candidate: &Self,
        elevation: worth_relational::facade::identity::EntityId,
    ) -> bool {
        Self::matches_elevated_request(self, candidate, elevation)
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn matches_use_for_test(&self, candidate: &Self) -> bool {
        Self::matches_use(self, candidate)
    }
}

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn action_mut_for_test(&mut self) -> &mut AspectValue {
        &mut self.action
    }
}

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn context_mut_for_test(&mut self) -> &mut RetainedContext {
        &mut self.context
    }
}

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn related_mut_for_test(
        &mut self,
    ) -> &mut Option<worth_relational::facade::identity::EntityId> {
        &mut self.related
    }
}

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn new_for_test(
        _codec_funding: Option<(
            Arc<crate::domain_computation::primary_graph::workflow::actor::ActorCodecFunding>,
            Arc<crate::domain_computation::primary_graph::workflow::actor::ActorCodecFunding>,
        )>,
        resource_entity: Arc<str>,
        elevation: Option<worth_relational::facade::identity::EntityId>,
        action: AspectValue,
        purpose: AspectValue,
        related_relation: Option<ApplicationCapabilityRelationBinding>,
        related: Option<worth_relational::facade::identity::EntityId>,
        field: Option<AspectValue>,
        magnitude: Option<AspectValue>,
        cardinality: u32,
        context_name: Arc<str>,
        context_type: Arc<str>,
        context: RetainedContext,
    ) -> Self {
        Self {
            _codec_funding,
            resource_entity,
            elevation,
            action,
            purpose,
            related_relation,
            related,
            field,
            magnitude,
            cardinality,
            context_name,
            context_type,
            context,
        }
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn new_for_test(
        recorded_actor: Option<Arc<crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowRecordedActorObservation>>,
        capability_identity: [u8; 32],
        principal: worth_relational::facade::identity::EntityId,
        resource: worth_relational::facade::identity::EntityId,
        data: Arc<RetainedCapabilityRequestData>,
    ) -> Self {
        Self {
            recorded_actor,
            capability_identity,
            principal,
            resource,
            data,
        }
    }
}
