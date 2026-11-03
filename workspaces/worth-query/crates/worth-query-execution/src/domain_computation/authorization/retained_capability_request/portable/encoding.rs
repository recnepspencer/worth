//! Borrowed view of the original admitted request; JSON bytes own their payer.
use super::*;
use crate::domain_computation::primary_graph::workflow::actor::encoding::{self, FundedActorText};

#[derive(Serialize)]
struct Request<'a> {
    version: u8,
    recorded_actor: RecordedActor<'a>,
    capability_identity: [u8; 32],
    principal: EntityId,
    resource: EntityId,
    resource_entity: &'a str,
    elevation: Option<EntityId>,
    action: &'a AspectValue,
    purpose: &'a AspectValue,
    related_relation: Option<Relation<'a>>,
    related: Option<EntityId>,
    field: &'a Option<AspectValue>,
    magnitude: &'a Option<AspectValue>,
    cardinality: u32,
    context_name: &'a str,
    context_type: &'a str,
    context: Context<'a>,
}

#[derive(Serialize)]
enum RecordedActor<'a> {
    NotApplicable,
    Native(&'a str),
}

#[derive(Serialize)]
struct Relation<'a> {
    relation: &'a str,
    from: &'a str,
    to: &'a str,
}

struct Context<'a>(&'a super::super::context::RetainedContext);
impl Serialize for Context<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter())
    }
}

impl WorthQueryRetainedCapabilityRequest {
    pub(in crate::domain_computation) fn encode_workflow_approval_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<FundedActorText, WorkflowApprovalRequestEncodingDenial> {
        use WorkflowApprovalRequestEncodingDenial::{InvalidRequest, NonPortableValue};
        // Both native and decoded context owners guarantee unique ordered keys.
        if self.resource_entity.is_empty()
            || self.context_name.is_empty()
            || self.context_type.is_empty()
            || self.related_relation.is_some() != self.related.is_some()
            || self.related_relation.as_ref().is_some_and(|relation| {
                relation.relation().is_empty()
                    || relation.from().is_empty()
                    || relation.to().is_empty()
            })
        {
            return Err(InvalidRequest);
        }
        if !portable_value(&self.action)
            || !portable_value(&self.purpose)
            || self
                .field
                .as_ref()
                .is_some_and(|value| !portable_value(value))
            || self
                .magnitude
                .as_ref()
                .is_some_and(|value| !portable_value(value))
        {
            return Err(NonPortableValue);
        }
        let actor = self
            .recorded_actor
            .as_ref()
            .map(|actor| actor.encode(execution, MAXIMUM_PORTABLE_BYTES))
            .transpose()?;
        encoding::encode(
            execution,
            &Request {
                version: PORTABLE_VERSION,
                recorded_actor: actor
                    .as_ref()
                    .map_or(RecordedActor::NotApplicable, |actor| {
                        RecordedActor::Native(actor.as_str())
                    }),
                capability_identity: self.capability_identity,
                principal: self.principal,
                resource: self.resource,
                resource_entity: &self.resource_entity,
                elevation: self.elevation,
                action: &self.action,
                purpose: &self.purpose,
                related_relation: self.related_relation.as_ref().map(|relation| Relation {
                    relation: relation.relation(),
                    from: relation.from(),
                    to: relation.to(),
                }),
                related: self.related,
                field: &self.field,
                magnitude: &self.magnitude,
                cardinality: self.cardinality,
                context_name: &self.context_name,
                context_type: &self.context_type,
                context: Context(&self.context),
            },
            MAXIMUM_PORTABLE_BYTES,
        )
    }
}
