use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRelationBinding, ApplicationCapabilityRequestProjection,
};

use super::operation_progression::{
    WorthQueryCapabilityContextKey, WorthQueryResolvedCapabilityRequest,
};

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryRetainedCapabilityRequest {
    capability_identity: [u8; 32],
    principal: worth_relational::facade::identity::EntityId,
    resource: worth_relational::facade::identity::EntityId,
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
    context: BTreeMap<WorthQueryCapabilityContextKey, worth_relational::facade::identity::EntityId>,
}

impl WorthQueryRetainedCapabilityRequest {
    pub(super) fn for_delegation_parent(
        &self,
        transition: &super::delegation_admission::observation::ObservedDelegationTransition,
    ) -> Self {
        let mut request = self.clone();
        request.principal = transition.grantor();
        request
    }

    pub(in crate::domain_computation::authorization) const fn capability_identity(
        &self,
    ) -> [u8; 32] {
        self.capability_identity
    }
    pub(in crate::domain_computation::authorization) const fn principal(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.principal
    }
    pub(in crate::domain_computation::authorization) const fn resource(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.resource
    }
    pub(in crate::domain_computation::authorization) fn resource_entity(&self) -> &str {
        &self.resource_entity
    }
    pub(in crate::domain_computation::authorization) const fn elevation(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.elevation
    }
    pub(in crate::domain_computation::authorization) const fn action(&self) -> &AspectValue {
        &self.action
    }
    pub(in crate::domain_computation::authorization) const fn purpose(&self) -> &AspectValue {
        &self.purpose
    }
    pub(in crate::domain_computation::authorization) const fn related_relation(
        &self,
    ) -> Option<&ApplicationCapabilityRelationBinding> {
        self.related_relation.as_ref()
    }
    pub(in crate::domain_computation::authorization) const fn related(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.related
    }
    pub(in crate::domain_computation::authorization) const fn field(&self) -> Option<&AspectValue> {
        self.field.as_ref()
    }
    pub(in crate::domain_computation::authorization) const fn magnitude(
        &self,
    ) -> Option<&AspectValue> {
        self.magnitude.as_ref()
    }
    pub(in crate::domain_computation::authorization) const fn cardinality(&self) -> u32 {
        self.cardinality
    }
    pub(in crate::domain_computation::authorization) fn context_name(&self) -> &str {
        &self.context_name
    }
    pub(in crate::domain_computation::authorization) fn context_type(&self) -> &str {
        &self.context_type
    }
    pub(in crate::domain_computation::authorization) const fn context(
        &self,
    ) -> &BTreeMap<WorthQueryCapabilityContextKey, worth_relational::facade::identity::EntityId>
    {
        &self.context
    }

    pub(in crate::domain_computation) fn workflow_subject(
        &self,
        selector: &worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
        match selector {
            ApplicationWorkflowSubjectSelector::Resource => Some(self.resource),
            ApplicationWorkflowSubjectSelector::Related => self.related,
            ApplicationWorkflowSubjectSelector::Context(slot) => self
                .context
                .get(&WorthQueryCapabilityContextKey::from_slot(slot))
                .copied(),
        }
    }

    pub(in crate::domain_computation::authorization) fn matches_elevated_request(
        &self,
        candidate: &Self,
        elevation: worth_relational::facade::identity::EntityId,
    ) -> bool {
        candidate.capability_identity == self.capability_identity
            && candidate.principal == self.principal
            && candidate.resource == self.resource
            && candidate.resource_entity == self.resource_entity
            && candidate.elevation == Some(elevation)
            && candidate.action == self.action
            && candidate.purpose == self.purpose
            && candidate.related_relation == self.related_relation
            && candidate.related == self.related
            && candidate.field == self.field
            && candidate.magnitude == self.magnitude
            && candidate.cardinality == self.cardinality
            && candidate.context_name == self.context_name
            && candidate.context_type == self.context_type
            && candidate.context == self.context
    }

    pub(super) fn capture<Schema, Scope, Context>(
        capability_identity: [u8; 32],
        principal: worth_relational::facade::identity::EntityId,
        projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
        resolved: &WorthQueryResolvedCapabilityRequest<Schema, Scope>,
    ) -> Self {
        Self {
            capability_identity,
            principal,
            resource: resolved.resource_entity_id(),
            resource_entity: Arc::from(projection.resource().entity()),
            elevation: resolved.elevation(),
            action: projection.action().clone(),
            purpose: projection.purpose().clone(),
            related_relation: projection
                .related()
                .map(|related| related.relation().clone()),
            related: resolved.related(),
            field: projection.field_value().cloned(),
            magnitude: projection.magnitude_value().cloned(),
            cardinality: projection.cardinality_value(),
            context_name: Arc::from(projection.context_value().context()),
            context_type: Arc::from(projection.context_value().context_type()),
            context: resolved.retained_context(),
        }
    }
}
