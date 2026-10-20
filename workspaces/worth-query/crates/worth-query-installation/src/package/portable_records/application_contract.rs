use std::cmp::Ordering;
use std::collections::BTreeSet;

use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectKey, AspectMask, CanonicalFieldPath, FieldKey,
    ProjectionMask,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use super::application_contract_parts::{
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableExternalEffectContractParts, WorthQueryPortableNativeAspectContractParts,
};
use super::WorthQueryPortableInstalledReconciliationProcedureRecord;

mod operation;
mod operation_body;
mod reconstruction_work;
pub use operation::WorthQueryPortableApplicationOperationContractRecord;

/// Runtime-neutral native aspect contract retained by package validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableNativeAspectContractRecord {
    schema: String,
    entity: String,
    aspect: AspectKey,
    contract: AspectContract,
    fields: BTreeSet<FieldKey>,
    binding: AspectBinding,
}

impl WorthQueryPortableNativeAspectContractRecord {
    pub fn from_untrusted_parts(parts: WorthQueryPortableNativeAspectContractParts) -> Self {
        Self {
            schema: parts.schema,
            entity: parts.entity,
            aspect: parts.aspect,
            contract: parts.contract,
            fields: parts.fields,
            binding: parts.binding,
        }
    }

    pub(crate) fn new(
        schema: String,
        entity: String,
        aspect: AspectKey,
        contract: AspectContract,
        fields: BTreeSet<FieldKey>,
        binding: AspectBinding,
    ) -> Self {
        Self {
            schema,
            entity,
            aspect,
            contract,
            fields,
            binding,
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    #[cfg(test)]
    pub(crate) fn replace_schema_for_test(&mut self, schema: impl Into<String>) {
        self.schema = schema.into();
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub const fn aspect(&self) -> &AspectKey {
        &self.aspect
    }

    pub const fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = &FieldKey> {
        self.fields.iter()
    }

    pub(crate) fn field(&self, field: &str) -> Option<&FieldKey> {
        self.fields
            .iter()
            .find(|candidate| candidate.as_str() == field)
    }

    pub(crate) fn retained_fields(&self) -> BTreeSet<FieldKey> {
        self.fields.clone()
    }

    pub const fn binding(&self) -> &AspectBinding {
        &self.binding
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableOperationGraphReadScope {
    Entity {
        schema: String,
        entity: String,
    },
    NativeProjection {
        schema: String,
        entity: String,
        aspect: AspectKey,
        contract: AspectContract,
        mask: AspectMask<ProjectionMask>,
    },
    Relation {
        schema: String,
        relation: String,
        from: String,
        to: String,
    },
}

impl WorthQueryPortableOperationGraphReadScope {
    pub fn schema(&self) -> &str {
        match self {
            Self::Entity { schema, .. }
            | Self::NativeProjection { schema, .. }
            | Self::Relation { schema, .. } => schema,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableOperationTouchScope {
    CreateEntity {
        schema: String,
        entity: String,
    },
    DeleteEntity {
        schema: String,
        entity: String,
    },
    WriteField {
        schema: String,
        entity: String,
        contract: AspectContract,
        field_path: CanonicalFieldPath,
    },
    LinkRelation {
        schema: String,
        relation: String,
        from: String,
        to: String,
    },
    UnlinkRelation {
        schema: String,
        relation: String,
        from: String,
        to: String,
    },
}

impl WorthQueryPortableOperationTouchScope {
    pub fn schema(&self) -> &str {
        match self {
            Self::CreateEntity { schema, .. }
            | Self::DeleteEntity { schema, .. }
            | Self::WriteField { schema, .. }
            | Self::LinkRelation { schema, .. }
            | Self::UnlinkRelation { schema, .. } => schema,
        }
    }

    pub(crate) fn canonical_order(left: &Self, right: &Self) -> Ordering {
        portable_touch_rank(left)
            .cmp(&portable_touch_rank(right))
            .then_with(|| left.schema().cmp(right.schema()))
            .then_with(|| match (left, right) {
                (
                    Self::CreateEntity { entity: left, .. }
                    | Self::DeleteEntity { entity: left, .. },
                    Self::CreateEntity { entity: right, .. }
                    | Self::DeleteEntity { entity: right, .. },
                ) => left.cmp(right),
                (
                    Self::WriteField {
                        entity: left_entity,
                        contract: left_contract,
                        field_path: left_field,
                        ..
                    },
                    Self::WriteField {
                        entity: right_entity,
                        contract: right_contract,
                        field_path: right_field,
                        ..
                    },
                ) => left_entity
                    .cmp(right_entity)
                    .then_with(|| {
                        left_contract
                            .key()
                            .as_str()
                            .cmp(right_contract.key().as_str())
                    })
                    .then_with(|| left_field.cmp(right_field)),
                (
                    Self::LinkRelation {
                        relation: left_relation,
                        from: left_from,
                        to: left_to,
                        ..
                    }
                    | Self::UnlinkRelation {
                        relation: left_relation,
                        from: left_from,
                        to: left_to,
                        ..
                    },
                    Self::LinkRelation {
                        relation: right_relation,
                        from: right_from,
                        to: right_to,
                        ..
                    }
                    | Self::UnlinkRelation {
                        relation: right_relation,
                        from: right_from,
                        to: right_to,
                        ..
                    },
                ) => left_relation
                    .cmp(right_relation)
                    .then_with(|| left_from.cmp(right_from))
                    .then_with(|| left_to.cmp(right_to)),
                _ => Ordering::Equal,
            })
    }
}

fn portable_touch_rank(scope: &WorthQueryPortableOperationTouchScope) -> u8 {
    match scope {
        WorthQueryPortableOperationTouchScope::CreateEntity { .. } => 0,
        WorthQueryPortableOperationTouchScope::DeleteEntity { .. } => 1,
        WorthQueryPortableOperationTouchScope::WriteField { .. } => 2,
        WorthQueryPortableOperationTouchScope::LinkRelation { .. } => 3,
        WorthQueryPortableOperationTouchScope::UnlinkRelation { .. } => 4,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableExternalEffectContractRecord {
    correlation_family: WorthQueryExternalEffectCorrelationFamily,
    effect: String,
    payload_type: WorthQueryPortableTypeIdentity,
    protocol: ApplicationExternalEffectProtocol,
    maximum_payload_bytes: u64,
}

impl WorthQueryPortableExternalEffectContractRecord {
    pub fn from_untrusted_parts(parts: WorthQueryPortableExternalEffectContractParts) -> Self {
        Self {
            correlation_family: parts.correlation_family,
            effect: parts.effect,
            payload_type: parts.payload_type,
            protocol: parts.protocol,
            maximum_payload_bytes: parts.maximum_payload_bytes,
        }
    }

    pub(crate) fn new(
        correlation_family: WorthQueryExternalEffectCorrelationFamily,
        effect: String,
        payload_type: WorthQueryPortableTypeIdentity,
        protocol: ApplicationExternalEffectProtocol,
        maximum_payload_bytes: u64,
    ) -> Self {
        Self {
            correlation_family,
            effect,
            payload_type,
            protocol,
            maximum_payload_bytes,
        }
    }

    pub const fn correlation_family(&self) -> &WorthQueryExternalEffectCorrelationFamily {
        &self.correlation_family
    }

    pub fn effect(&self) -> &str {
        &self.effect
    }

    pub const fn payload_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.payload_type
    }

    pub const fn protocol(&self) -> &ApplicationExternalEffectProtocol {
        &self.protocol
    }

    pub const fn maximum_payload_bytes(&self) -> u64 {
        self.maximum_payload_bytes
    }
}
