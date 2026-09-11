use crate::application_schema::{
    ApplicationEffectRef, ApplicationFieldUnit, ApplicationRetainedEffectBinding,
    ApplicationScalarValueBinding, ApplicationStructuredValueBinding, EqualityPredicate, ReadOnly,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

use super::{ApplicationQueryMarkerIdentity, ApplicationQueryResultFieldRef};

/// Domain-owned interpretation of one committed effect as a live-query cause.
///
/// The binding type is identity-bearing query meaning. Query invokes these
/// functions only after installation has matched the exact binding, effect,
/// payload, scope selector, and target selector.
pub trait ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>:
    WorthQueryPortableType + 'static
{
    type Effect;
    type PayloadBinding: ApplicationRetainedEffectBinding;
    type ScopeIdentityBinding: ApplicationScalarValueBinding;
    type TargetIdentityBinding: ApplicationScalarValueBinding;

    fn effect() -> ApplicationEffectRef<
        Schema,
        Self::Effect,
        <Self::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    >;
    fn scope_identity(
        payload: &<Self::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    ) -> <Self::ScopeIdentityBinding as ApplicationScalarValueBinding>::Value;
    fn target_identity(
        payload: &<Self::PayloadBinding as ApplicationStructuredValueBinding>::Value,
    ) -> <Self::TargetIdentityBinding as ApplicationScalarValueBinding>::Value;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryLiveResourceContract {
    maximum_buffered_causes: u64,
    maximum_work_per_delivery: u64,
    maximum_retained_payload_bytes: u64,
}

impl ApplicationQueryLiveResourceContract {
    pub const fn bounded(
        maximum_buffered_causes: u64,
        maximum_work_per_delivery: u64,
        maximum_retained_payload_bytes: u64,
    ) -> Self {
        Self {
            maximum_buffered_causes,
            maximum_work_per_delivery,
            maximum_retained_payload_bytes,
        }
    }

    pub const fn maximum_buffered_causes(self) -> u64 {
        self.maximum_buffered_causes
    }

    pub const fn maximum_work_per_delivery(self) -> u64 {
        self.maximum_work_per_delivery
    }

    pub const fn maximum_retained_payload_bytes(self) -> u64 {
        self.maximum_retained_payload_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryLiveCauseContract {
    binding_type: WorthQueryPortableTypeIdentity,
    effect: String,
    payload_type: WorthQueryPortableTypeIdentity,
    scope_slot_type: WorthQueryPortableTypeIdentity,
    scope_field: (String, String, String),
    scope_value_type: WorthQueryPortableTypeIdentity,
    target_slot_type: WorthQueryPortableTypeIdentity,
    target_field: (String, String, String),
    target_value_type: WorthQueryPortableTypeIdentity,
    resources: ApplicationQueryLiveResourceContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryLiveCauseParts {
    pub binding_type: WorthQueryPortableTypeIdentity,
    pub effect: String,
    pub payload_type: WorthQueryPortableTypeIdentity,
    pub scope_slot_type: WorthQueryPortableTypeIdentity,
    pub scope_entity: String,
    pub scope_aspect: String,
    pub scope_field: String,
    pub scope_value_type: WorthQueryPortableTypeIdentity,
    pub target_slot_type: WorthQueryPortableTypeIdentity,
    pub target_entity: String,
    pub target_aspect: String,
    pub target_field: String,
    pub target_value_type: WorthQueryPortableTypeIdentity,
    pub resources: ApplicationQueryLiveResourceContract,
}

impl ApplicationQueryLiveCauseContract {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryLiveCauseParts) -> Self {
        Self {
            binding_type: parts.binding_type,
            effect: parts.effect,
            payload_type: parts.payload_type,
            scope_slot_type: parts.scope_slot_type,
            scope_field: (parts.scope_entity, parts.scope_aspect, parts.scope_field),
            scope_value_type: parts.scope_value_type,
            target_slot_type: parts.target_slot_type,
            target_field: (parts.target_entity, parts.target_aspect, parts.target_field),
            target_value_type: parts.target_value_type,
            resources: parts.resources,
        }
    }

    pub const fn binding_type(&self) -> &str {
        self.binding_type.as_str()
    }

    pub fn binding_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.binding_type.clone()
    }

    pub fn effect(&self) -> &str {
        &self.effect
    }

    pub const fn payload_type(&self) -> &str {
        self.payload_type.as_str()
    }

    pub fn payload_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.payload_type.clone()
    }

    pub const fn scope_slot_type(&self) -> &str {
        self.scope_slot_type.as_str()
    }

    pub fn scope_slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.scope_slot_type.clone()
    }

    pub fn scope_field(&self) -> (&str, &str, &str) {
        (
            &self.scope_field.0,
            &self.scope_field.1,
            &self.scope_field.2,
        )
    }

    pub const fn scope_value_type(&self) -> &str {
        self.scope_value_type.as_str()
    }

    pub const fn target_slot_type(&self) -> &str {
        self.target_slot_type.as_str()
    }

    pub fn target_slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.target_slot_type.clone()
    }

    pub fn target_field(&self) -> (&str, &str, &str) {
        (
            &self.target_field.0,
            &self.target_field.1,
            &self.target_field.2,
        )
    }

    pub const fn target_value_type(&self) -> &str {
        self.target_value_type.as_str()
    }

    pub const fn resources(&self) -> ApplicationQueryLiveResourceContract {
        self.resources
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn typed<
        Schema,
        Query,
        Scope,
        Target,
        Binding,
        ScopeSlot,
        ScopeAspect,
        ScopeField,
        ScopeUnit,
        TargetSlot,
        TargetAspect,
        TargetField,
        TargetUnit,
    >(
        scope_identity: ApplicationQueryResultFieldRef<
            Query,
            ScopeSlot,
            Schema,
            Scope,
            ScopeAspect,
            ScopeField,
            <Binding::ScopeIdentityBinding as ApplicationScalarValueBinding>::Value,
            ReadOnly,
            EqualityPredicate,
            ScopeUnit,
        >,
        target_identity: ApplicationQueryResultFieldRef<
            Query,
            TargetSlot,
            Schema,
            Target,
            TargetAspect,
            TargetField,
            <Binding::TargetIdentityBinding as ApplicationScalarValueBinding>::Value,
            ReadOnly,
            EqualityPredicate,
            TargetUnit,
        >,
        resources: ApplicationQueryLiveResourceContract,
    ) -> Self
    where
        Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
        ScopeField: crate::application_schema::RequiredApplicationFieldValue<
            Value = <Binding::ScopeIdentityBinding as ApplicationScalarValueBinding>::Value,
            Binding = Binding::ScopeIdentityBinding,
        >,
        TargetField: crate::application_schema::RequiredApplicationFieldValue<
            Value = <Binding::TargetIdentityBinding as ApplicationScalarValueBinding>::Value,
            Binding = Binding::TargetIdentityBinding,
        >,
        ScopeUnit: ApplicationFieldUnit,
        TargetUnit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        ScopeSlot: WorthQueryPortableType,
        TargetSlot: WorthQueryPortableType,
    {
        Self {
            binding_type: Binding::PORTABLE_TYPE_IDENTITY,
            effect: Binding::effect().name().to_owned(),
            payload_type: Binding::PayloadBinding::IDENTITY,
            scope_slot_type: scope_identity.slot_key().slot_identity(),
            scope_field: (
                scope_identity.entity().to_owned(),
                scope_identity.aspect().to_owned(),
                scope_identity.field().to_owned(),
            ),
            scope_value_type: Binding::ScopeIdentityBinding::IDENTITY,
            target_slot_type: target_identity.slot_key().slot_identity(),
            target_field: (
                target_identity.entity().to_owned(),
                target_identity.aspect().to_owned(),
                target_identity.field().to_owned(),
            ),
            target_value_type: Binding::TargetIdentityBinding::IDENTITY,
            resources,
        }
    }
}
