use std::marker::PhantomData;

use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationRelationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    EqualityCapable, EqualityPosture, WritePosture,
};

use super::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityContextEntitySlotRef,
    ApplicationCapabilityContextRef, ApplicationCapabilityGovernedInputIdentity,
    ApplicationCapabilityRelationBinding,
};

/// Application-owned projection of one exact operation input into the
/// request-varying dimensions constrained by an installed capability.
///
/// Implementations describe the input; they do not grant authority. Query
/// validates the projection against installed meaning and current graph truth.
pub trait ApplicationCapabilityRequest<Schema, Capability> {
    type Scope;
    type Context;

    /// Stable application-owned identity of governed input dimensions whose
    /// drift must make an idempotent retry inequivalent. Query retains and
    /// privately composes this identity with the installed operation and
    /// admitted scope; it is not capability authority or graph truth.
    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        None
    }

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    >;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationCapabilityRequestProjectionDenial {
    subject: String,
}

impl ApplicationCapabilityRequestProjectionDenial {
    pub fn input_variant(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

pub struct ApplicationCapabilityEntitySelector<Schema, Entity> {
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    scalar_family: ScalarAspectType,
    value_type: &'static str,
    value: AspectValue,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> Clone for ApplicationCapabilityEntitySelector<Schema, Entity> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            aspect: self.aspect,
            field: self.field,
            scalar_family: self.scalar_family,
            value_type: self.value_type,
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> ApplicationCapabilityEntitySelector<Schema, Entity> {
    pub fn new<Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Equality: EqualityPosture + EqualityCapable,
        Unit: ApplicationFieldUnit,
    {
        Self {
            entity: field.entity(),
            aspect: field.aspect(),
            field: field.field(),
            scalar_family: field.scalar_family(),
            value_type: field.value_type_name(),
            value: value.into_foundational_value(),
            _marker: PhantomData,
        }
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn value_type(&self) -> &'static str {
        self.value_type
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

pub struct ApplicationCapabilityRelatedEntitySelector<Schema> {
    relation: ApplicationCapabilityRelationBinding,
    selector: ErasedApplicationCapabilityEntitySelector,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> ApplicationCapabilityRelatedEntitySelector<Schema> {
    pub fn new<Relation, From, Entity>(
        relation: ApplicationRelationRef<Schema, Relation, From, Entity>,
        selector: ApplicationCapabilityEntitySelector<Schema, Entity>,
    ) -> Self {
        Self {
            relation: ApplicationCapabilityRelationBinding::from_reference(relation),
            selector: selector.erase(),
            _schema: PhantomData,
        }
    }

    pub const fn relation(&self) -> &ApplicationCapabilityRelationBinding {
        &self.relation
    }

    pub const fn selector(&self) -> &ErasedApplicationCapabilityEntitySelector {
        &self.selector
    }
}

pub struct ApplicationCapabilityRequestContext<Schema, Context> {
    context: &'static str,
    context_identity: crate::portable_identity::WorthQueryPortableTypeIdentity,
    entities: Vec<ApplicationCapabilityContextEntitySelector>,
    _marker: PhantomData<fn() -> (Schema, Context)>,
}

impl<Schema, Context> ApplicationCapabilityRequestContext<Schema, Context> {
    pub fn new(context: ApplicationCapabilityContextRef<Schema, Context>) -> Self {
        Self {
            context: context.name(),
            context_identity: context.marker_identity(),
            entities: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn entity<Slot, Entity>(
        mut self,
        slot: ApplicationCapabilityContextEntitySlotRef<Schema, Context, Slot, Entity>,
        selector: ApplicationCapabilityEntitySelector<Schema, Entity>,
    ) -> Self {
        self.entities
            .push(ApplicationCapabilityContextEntitySelector {
                slot: ApplicationCapabilityContextEntitySlotBinding::from_reference(slot),
                selector: selector.erase(),
            });
        self
    }

    pub const fn context(&self) -> &'static str {
        self.context
    }

    pub const fn context_type(&self) -> &str {
        self.context_identity.as_str()
    }

    pub fn context_identity(&self) -> crate::portable_identity::WorthQueryPortableTypeIdentity {
        self.context_identity.clone()
    }

    pub fn entities(&self) -> &[ApplicationCapabilityContextEntitySelector] {
        &self.entities
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationCapabilityContextEntitySelector {
    slot: ApplicationCapabilityContextEntitySlotBinding,
    selector: ErasedApplicationCapabilityEntitySelector,
}

impl ApplicationCapabilityContextEntitySelector {
    pub const fn slot(&self) -> &ApplicationCapabilityContextEntitySlotBinding {
        &self.slot
    }

    pub const fn selector(&self) -> &ErasedApplicationCapabilityEntitySelector {
        &self.selector
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasedApplicationCapabilityEntitySelector {
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    scalar_family: ScalarAspectType,
    value_type: &'static str,
    value: AspectValue,
}

impl ErasedApplicationCapabilityEntitySelector {
    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn value_type(&self) -> &'static str {
        self.value_type
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

impl<Schema, Entity> ApplicationCapabilityEntitySelector<Schema, Entity> {
    pub(super) fn erase(self) -> ErasedApplicationCapabilityEntitySelector {
        ErasedApplicationCapabilityEntitySelector {
            entity: self.entity,
            aspect: self.aspect,
            field: self.field,
            scalar_family: self.scalar_family,
            value_type: self.value_type,
            value: self.value,
        }
    }
}

pub struct ApplicationCapabilityRequestProjection<Schema, Scope, Context> {
    resource: ApplicationCapabilityEntitySelector<Schema, Scope>,
    elevation: Option<ErasedApplicationCapabilityEntitySelector>,
    action: AspectValue,
    purpose: AspectValue,
    related_entity: Option<ApplicationCapabilityRelatedEntitySelector<Schema>>,
    field: Option<AspectValue>,
    magnitude: Option<AspectValue>,
    cardinality: u32,
    context: ApplicationCapabilityRequestContext<Schema, Context>,
}

impl<Schema, Scope, Context> ApplicationCapabilityRequestProjection<Schema, Scope, Context> {
    pub fn new<ActionBinding, PurposeBinding>(
        resource: ApplicationCapabilityEntitySelector<Schema, Scope>,
        action: ApplicationEncodedScalarValue<ActionBinding>,
        purpose: ApplicationEncodedScalarValue<PurposeBinding>,
        context: ApplicationCapabilityRequestContext<Schema, Context>,
    ) -> Self
    where
        ActionBinding: ApplicationScalarValueBinding,
        PurposeBinding: ApplicationScalarValueBinding,
    {
        Self {
            resource,
            elevation: None,
            action: action.into_foundational_value(),
            purpose: purpose.into_foundational_value(),
            related_entity: None,
            field: None,
            magnitude: None,
            cardinality: 1,
            context,
        }
    }

    /// Selects the exact governed elevation record carried by this operation
    /// input. The selector is descriptive input; Query resolves and validates
    /// it against installed elevation meaning before authority can exist.
    pub fn elevation<Entity>(
        mut self,
        elevation: ApplicationCapabilityEntitySelector<Schema, Entity>,
    ) -> Self {
        self.elevation = Some(elevation.erase());
        self
    }

    pub fn related_entity(
        mut self,
        related: ApplicationCapabilityRelatedEntitySelector<Schema>,
    ) -> Self {
        self.related_entity = Some(related);
        self
    }

    pub fn field<Binding: ApplicationScalarValueBinding>(
        mut self,
        field: ApplicationEncodedScalarValue<Binding>,
    ) -> Self {
        self.field = Some(field.into_foundational_value());
        self
    }

    pub fn magnitude<Binding: ApplicationScalarValueBinding>(
        mut self,
        magnitude: ApplicationEncodedScalarValue<Binding>,
    ) -> Self {
        self.magnitude = Some(magnitude.into_foundational_value());
        self
    }

    pub const fn cardinality(mut self, cardinality: u32) -> Self {
        self.cardinality = cardinality;
        self
    }

    pub const fn resource(&self) -> &ApplicationCapabilityEntitySelector<Schema, Scope> {
        &self.resource
    }

    pub const fn elevation_selector(&self) -> Option<&ErasedApplicationCapabilityEntitySelector> {
        self.elevation.as_ref()
    }

    pub const fn action(&self) -> &AspectValue {
        &self.action
    }

    pub const fn purpose(&self) -> &AspectValue {
        &self.purpose
    }

    pub const fn related(&self) -> Option<&ApplicationCapabilityRelatedEntitySelector<Schema>> {
        self.related_entity.as_ref()
    }

    pub const fn field_value(&self) -> Option<&AspectValue> {
        self.field.as_ref()
    }

    pub const fn magnitude_value(&self) -> Option<&AspectValue> {
        self.magnitude.as_ref()
    }

    pub const fn cardinality_value(&self) -> u32 {
        self.cardinality
    }

    pub const fn context_value(&self) -> &ApplicationCapabilityRequestContext<Schema, Context> {
        &self.context
    }
}
