//! Durable, bounded form of the exact request authorized at workflow approval.
mod context_inventory;

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRelationBinding,
    WorthQueryPortableApplicationCapabilityRelationBindingParts,
};
use worth_relational::facade::identity::EntityId;

use super::{WorthQueryCapabilityContextKey, WorthQueryRetainedCapabilityRequest};

const PORTABLE_VERSION: u8 = 2;
pub(in crate::domain_computation) const MAXIMUM_PORTABLE_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRequest<Entries = Vec<(WorthQueryCapabilityContextKey, EntityId)>> {
    version: u8,
    recorded_actor: RecordedActorRequest,
    capability_identity: [u8; 32],
    principal: EntityId,
    resource: EntityId,
    resource_entity: String,
    elevation: Option<EntityId>,
    action: AspectValue,
    purpose: AspectValue,
    related_relation: Option<PortableRelation>,
    related: Option<EntityId>,
    field: Option<AspectValue>,
    magnitude: Option<AspectValue>,
    cardinality: u32,
    context_name: String,
    context_type: String,
    context: Entries,
}

#[derive(Serialize, Deserialize)]
enum RecordedActorRequest {
    NotApplicable,
    Native(String),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRelation {
    relation: String,
    from: String,
    to: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorkflowApprovalRequestEncodingDenial {
    TooLarge,
    NonPortableValue,
    InvalidRequest,
}

#[derive(Debug)]
pub(in crate::domain_computation) enum WorkflowApprovalRequestDecodingDenial {
    InvalidRequest,
    Actor(Box<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>),
}
impl From<()> for WorkflowApprovalRequestDecodingDenial {
    fn from(_: ()) -> Self {
        Self::InvalidRequest
    }
}

impl WorthQueryRetainedCapabilityRequest {
    pub(in crate::domain_computation) fn encode_workflow_approval_request(
        &self,
    ) -> Result<String, WorkflowApprovalRequestEncodingDenial> {
        let relation = self.related_relation.as_ref().map(|binding| {
            let parts = binding.parts();
            PortableRelation {
                relation: parts.relation,
                from: parts.from,
                to: parts.to,
            }
        });
        let portable = PortableRequest {
            version: PORTABLE_VERSION,
            recorded_actor: match &self.recorded_actor {
                None => RecordedActorRequest::NotApplicable,
                Some(actor) => RecordedActorRequest::Native(
                    actor
                        .encode()
                        .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?,
                ),
            },
            capability_identity: self.capability_identity,
            principal: self.principal,
            resource: self.resource,
            resource_entity: self.resource_entity.to_string(),
            elevation: self.elevation,
            action: self.action.clone(),
            purpose: self.purpose.clone(),
            related_relation: relation,
            related: self.related,
            field: self.field.clone(),
            magnitude: self.magnitude.clone(),
            cardinality: self.cardinality,
            context_name: self.context_name.to_string(),
            context_type: self.context_type.to_string(),
            context: self
                .context
                .iter()
                .map(|(key, entity)| (key.clone(), *entity))
                .collect(),
        };
        validate_portable(&portable)?;
        let encoded = serde_json::to_string(&portable)
            .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?;
        (encoded.len() <= MAXIMUM_PORTABLE_BYTES)
            .then_some(encoded)
            .ok_or(WorkflowApprovalRequestEncodingDenial::TooLarge)
    }

    pub(in crate::domain_computation) fn decode_workflow_approval_request(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        encoded: &str,
    ) -> Result<Self, WorkflowApprovalRequestDecodingDenial> {
        if encoded.len() > MAXIMUM_PORTABLE_BYTES {
            return Err(WorkflowApprovalRequestDecodingDenial::InvalidRequest);
        }
        let text_bytes = encoded.len().checked_mul(3).ok_or(())?;
        let text_funding =
            crate::domain_computation::primary_graph::workflow::actor::fund_actor_codec(
                execution, text_bytes,
            )
            .map_err(|cause| WorkflowApprovalRequestDecodingDenial::Actor(Box::new(cause)))?;
        let inventory: PortableRequest<context_inventory::ContextCount> =
            serde_json::from_str(encoded).map_err(|_| ())?;
        let entries = inventory.context.0;
        drop(inventory);
        // serde JSON sequence has no length hint. Vec's nonzero-size initial
        // capacity is at most4 and growth is geometric. The decoded sorted
        // Vec itself is retained; no unquoted BTreeMap nodes are allocated.
        let slots = if entries == 0 {
            0
        } else {
            entries.checked_next_power_of_two().ok_or(())?.max(4)
        };
        let bytes = Some(slots)
            .and_then(|slots| {
                slots.checked_mul(std::mem::size_of::<(
                    WorthQueryCapabilityContextKey,
                    EntityId,
                )>())
            })
            .ok_or(())?;
        let context_funding =
            crate::domain_computation::primary_graph::workflow::actor::fund_actor_codec(
                execution, bytes,
            )
            .map_err(|cause| WorkflowApprovalRequestDecodingDenial::Actor(Box::new(cause)))?;
        let portable: PortableRequest = serde_json::from_str(encoded).map_err(|_| ())?;
        validate_portable(&portable).map_err(|_| ())?;
        let context = super::context::RetainedContext::decoded(portable.context).map_err(|_| ())?;
        let related_relation = portable.related_relation.map(|parts| {
            ApplicationCapabilityRelationBinding::from_untrusted_parts(
                WorthQueryPortableApplicationCapabilityRelationBindingParts {
                    relation: parts.relation,
                    from: parts.from,
                    to: parts.to,
                },
            )
        });
        Ok(Self {
            recorded_actor: match portable.recorded_actor {
                RecordedActorRequest::NotApplicable => None,
                RecordedActorRequest::Native(encoded) if !encoded.is_empty() => Some(Arc::new(
                    crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowRecordedActorObservation::decode(execution, &encoded)
                        .map_err(|cause| WorkflowApprovalRequestDecodingDenial::Actor(Box::new(cause)))?)),
                RecordedActorRequest::Native(_) => return Err(WorkflowApprovalRequestDecodingDenial::InvalidRequest),
            },
            principal: portable.principal,
            capability_identity: portable.capability_identity,
            resource: portable.resource,
            data: Arc::new(super::RetainedCapabilityRequestData {
            _codec_funding: Some((text_funding, context_funding)),
            resource_entity: Arc::from(portable.resource_entity),
            elevation: portable.elevation,
            action: portable.action,
            purpose: portable.purpose,
            related_relation,
            related: portable.related,
            field: portable.field,
            magnitude: portable.magnitude,
            cardinality: portable.cardinality,
            context_name: Arc::from(portable.context_name),
            context_type: Arc::from(portable.context_type),
            context,
            }),
        })
    }
}

fn validate_portable(
    portable: &PortableRequest,
) -> Result<(), WorkflowApprovalRequestEncodingDenial> {
    use WorkflowApprovalRequestEncodingDenial::{InvalidRequest, NonPortableValue};
    if portable.version != PORTABLE_VERSION
        || portable.resource_entity.is_empty()
        || portable.context_name.is_empty()
        || portable.context_type.is_empty()
        || portable.related_relation.is_some() != portable.related.is_some()
        || portable.related_relation.as_ref().is_some_and(|relation| {
            relation.relation.is_empty() || relation.from.is_empty() || relation.to.is_empty()
        })
        || portable
            .context
            .iter()
            .map(|(key, _)| key)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != portable.context.len()
    {
        return Err(InvalidRequest);
    }
    if !portable_value(&portable.action)
        || !portable_value(&portable.purpose)
        || portable
            .field
            .as_ref()
            .is_some_and(|value| !portable_value(value))
        || portable
            .magnitude
            .as_ref()
            .is_some_and(|value| !portable_value(value))
    {
        return Err(NonPortableValue);
    }
    Ok(())
}

fn portable_value(value: &AspectValue) -> bool {
    !matches!(
        value,
        AspectValue::String(InternedString::Symbol(_))
            | AspectValue::Bytes(_)
            | AspectValue::ContentRef(_)
    )
}

#[cfg(test)]
mod tests;
