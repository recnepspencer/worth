//! Durable, bounded form of the exact request authorized at workflow approval.

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

const PORTABLE_VERSION: u8 = 1;
pub(in crate::domain_computation) const MAXIMUM_PORTABLE_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRequest {
    version: u8,
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
    context: Vec<(WorthQueryCapabilityContextKey, EntityId)>,
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
        encoded: &str,
    ) -> Result<Self, ()> {
        if encoded.len() > MAXIMUM_PORTABLE_BYTES {
            return Err(());
        }
        let portable: PortableRequest = serde_json::from_str(encoded).map_err(|_| ())?;
        validate_portable(&portable).map_err(|_| ())?;
        let count = portable.context.len();
        let context = portable.context.into_iter().collect::<BTreeMap<_, _>>();
        if context.len() != count {
            return Err(());
        }
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
            capability_identity: portable.capability_identity,
            principal: portable.principal,
            resource: portable.resource,
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
mod tests {
    use super::*;

    #[test]
    fn approval_request_round_trip_preserves_every_authorization_axis() {
        let mut request = WorthQueryRetainedCapabilityRequest {
            capability_identity: [7; 32],
            principal: EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                0,
            ),
            resource: EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                2,
                0,
            ),
            resource_entity: Arc::from("resource"),
            elevation: Some(EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                3,
                0,
            )),
            action: AspectValue::UInt64(4),
            purpose: AspectValue::UInt64(5),
            related_relation: Some(ApplicationCapabilityRelationBinding::from_untrusted_parts(
                WorthQueryPortableApplicationCapabilityRelationBindingParts {
                    relation: "related".into(),
                    from: "resource".into(),
                    to: "subject".into(),
                },
            )),
            related: Some(EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                6,
                0,
            )),
            field: Some(AspectValue::UInt64(7)),
            magnitude: Some(AspectValue::UInt64(8)),
            cardinality: 9,
            context_name: Arc::from("context"),
            context_type: Arc::from("context-type"),
            context: BTreeMap::new(),
        };
        let context_key = serde_json::from_str::<WorthQueryCapabilityContextKey>(
            r#"{"context":"context","context_type":"context-type","slot":"peer","slot_type":"entity","entity":"subject"}"#,
        )
        .unwrap();
        request.context.insert(
            context_key,
            EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                10,
                0,
            ),
        );
        let encoded = request.encode_workflow_approval_request().unwrap();
        let restored =
            WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&encoded)
                .unwrap();
        assert!(request.matches_elevated_request(&restored, request.elevation.unwrap()));
        let mut inconsistent = request.clone();
        inconsistent.related = None;
        assert_eq!(
            inconsistent.encode_workflow_approval_request(),
            Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest)
        );
        let mut duplicated: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        let context = duplicated["context"].as_array_mut().unwrap();
        let first = context[0].clone();
        context.push(first);
        assert!(
            WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                &duplicated.to_string()
            )
            .is_err()
        );
        let mut nonportable = request;
        nonportable.action = AspectValue::String(InternedString::Symbol(
            worth_foundational::facade::Symbol(1),
        ));
        assert!(nonportable.encode_workflow_approval_request().is_err());
    }

    #[test]
    fn approval_request_rejects_unknown_or_oversized_forms() {
        assert!(
            WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request("{}").is_err()
        );
        assert!(
            WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                &"x".repeat(MAXIMUM_PORTABLE_BYTES + 1)
            )
            .is_err()
        );
    }
}
