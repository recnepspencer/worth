use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationRelationRef, DeclaredApplicationFieldValue, WritePosture,
};
use worth_foundational::facade::{AspectValue, ScalarAspectType};

use super::{
    ApplicationCapabilityContextRef, ApplicationCapabilityCurrentnessDefinition,
    ApplicationCapabilityProvenanceRef,
};

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationCapabilityConstraintParts,
    WorthQueryPortableApplicationCapabilityDelegationParts,
    WorthQueryPortableApplicationCapabilityFieldBindingParts,
    WorthQueryPortableApplicationCapabilityRelationBindingParts,
    WorthQueryPortableApplicationCapabilityValueBindingParts,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityFieldBinding {
    entity: String,
    aspect: String,
    field: String,
    scalar_family: ScalarAspectType,
    value_type: String,
}

impl ApplicationCapabilityFieldBinding {
    pub fn from_reference<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        Self {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
            scalar_family: field.scalar_family(),
            value_type: field.value_type_name().to_string(),
        }
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub fn value_type(&self) -> &str {
        &self.value_type
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityValueBinding {
    field: ApplicationCapabilityFieldBinding,
    value: AspectValue,
}

impl ApplicationCapabilityValueBinding {
    pub fn new<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        Self {
            field: ApplicationCapabilityFieldBinding::from_reference(field),
            value: value.into_foundational_value(),
        }
    }

    pub const fn field(&self) -> &ApplicationCapabilityFieldBinding {
        &self.field
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityRelationBinding {
    relation: String,
    from: String,
    to: String,
}

impl ApplicationCapabilityRelationBinding {
    pub fn from_reference<Schema, Relation, From, To>(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        }
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityFieldDimension {
    NotApplicable,
    Bound(ApplicationCapabilityFieldBinding),
}

impl ApplicationCapabilityFieldDimension {
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }

    pub fn bound<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        Self::Bound(ApplicationCapabilityFieldBinding::from_reference(field))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityRelationDimension {
    NotApplicable,
    Bound(ApplicationCapabilityRelationBinding),
}

impl ApplicationCapabilityRelationDimension {
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }

    pub fn bound<Schema, Relation, From, To>(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::Bound(ApplicationCapabilityRelationBinding::from_reference(
            relation,
        ))
    }
}

pub type ApplicationCapabilityMagnitudeDimension = ApplicationCapabilityFieldDimension;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityCardinalityDimension {
    One,
    Many,
    Bounded(u32),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityTargetDefinition {
    action: ApplicationCapabilityValueBinding,
    resource: ApplicationCapabilityRelationBinding,
    relation: ApplicationCapabilityRelationDimension,
    field: ApplicationCapabilityFieldDimension,
    purpose: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityTargetDefinition {
    pub fn new(
        action: ApplicationCapabilityValueBinding,
        resource: ApplicationCapabilityRelationBinding,
        relation: ApplicationCapabilityRelationDimension,
        field: ApplicationCapabilityFieldDimension,
        purpose: ApplicationCapabilityValueBinding,
    ) -> Self {
        Self {
            action,
            resource,
            relation,
            field,
            purpose,
        }
    }

    pub fn action(&self) -> &ApplicationCapabilityValueBinding {
        &self.action
    }

    pub fn resource(&self) -> &ApplicationCapabilityRelationBinding {
        &self.resource
    }

    pub const fn relation(&self) -> &ApplicationCapabilityRelationDimension {
        &self.relation
    }

    pub const fn field(&self) -> &ApplicationCapabilityFieldDimension {
        &self.field
    }

    pub fn purpose(&self) -> &ApplicationCapabilityValueBinding {
        &self.purpose
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityConstraintDefinition {
    magnitude: ApplicationCapabilityMagnitudeDimension,
    cardinality: ApplicationCapabilityCardinalityDimension,
    currentness: ApplicationCapabilityCurrentnessDefinition,
    context: String,
    context_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
}

impl ApplicationCapabilityConstraintDefinition {
    pub fn new<Schema, Context>(
        magnitude: ApplicationCapabilityMagnitudeDimension,
        cardinality: ApplicationCapabilityCardinalityDimension,
        currentness: ApplicationCapabilityCurrentnessDefinition,
        context: ApplicationCapabilityContextRef<Schema, Context>,
    ) -> Self {
        Self {
            magnitude,
            cardinality,
            currentness,
            context: context.name().to_string(),
            context_type: context.marker_identity(),
        }
    }

    pub const fn magnitude(&self) -> &ApplicationCapabilityMagnitudeDimension {
        &self.magnitude
    }

    pub const fn cardinality(&self) -> ApplicationCapabilityCardinalityDimension {
        self.cardinality
    }

    pub const fn currentness(&self) -> &ApplicationCapabilityCurrentnessDefinition {
        &self.currentness
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn context_type(&self) -> &str {
        self.context_type.as_str()
    }

    pub fn context_identity(&self) -> crate::portable_identity::WorthQueryPortableTypeIdentity {
        self.context_type.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityDelegationDefinition {
    parent: ApplicationCapabilityRelationBinding,
    grantor: ApplicationCapabilityRelationBinding,
    grantee: ApplicationCapabilityRelationBinding,
    limit: ApplicationCapabilityFieldBinding,
    provenance: String,
    provenance_type: crate::portable_identity::WorthQueryPortableTypeIdentity,
    activation: Option<super::ApplicationCapabilityDelegationActivationDefinition>,
    revocation: Option<super::ApplicationCapabilityRevocationDefinition>,
}

impl ApplicationCapabilityDelegationDefinition {
    pub fn new<Schema, Provenance>(
        parent: ApplicationCapabilityRelationBinding,
        grantor: ApplicationCapabilityRelationBinding,
        grantee: ApplicationCapabilityRelationBinding,
        limit: ApplicationCapabilityFieldBinding,
        provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
    ) -> Self {
        Self {
            parent,
            grantor,
            grantee,
            limit,
            provenance: provenance.name().to_string(),
            provenance_type: provenance.marker_identity(),
            activation: None,
            revocation: None,
        }
    }

    pub fn with_activation(
        mut self,
        activation: super::ApplicationCapabilityDelegationActivationDefinition,
    ) -> Self {
        self.activation = Some(activation);
        self
    }

    pub fn with_revocation(
        mut self,
        revocation: super::ApplicationCapabilityRevocationDefinition,
    ) -> Self {
        self.revocation = Some(revocation);
        self
    }

    pub fn parent(&self) -> &ApplicationCapabilityRelationBinding {
        &self.parent
    }

    pub fn grantor(&self) -> &ApplicationCapabilityRelationBinding {
        &self.grantor
    }

    pub fn grantee(&self) -> &ApplicationCapabilityRelationBinding {
        &self.grantee
    }

    pub fn limit(&self) -> &ApplicationCapabilityFieldBinding {
        &self.limit
    }

    pub fn provenance(&self) -> &str {
        &self.provenance
    }

    pub fn provenance_type(&self) -> &str {
        self.provenance_type.as_str()
    }

    pub fn provenance_identity(&self) -> crate::portable_identity::WorthQueryPortableTypeIdentity {
        self.provenance_type.clone()
    }

    pub const fn activation(
        &self,
    ) -> Option<&super::ApplicationCapabilityDelegationActivationDefinition> {
        self.activation.as_ref()
    }

    pub const fn revocation(&self) -> Option<&super::ApplicationCapabilityRevocationDefinition> {
        self.revocation.as_ref()
    }
}
