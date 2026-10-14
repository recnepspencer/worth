//! Typed creation authoring. A created entity is not the admitted scope.

use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationEntityRef, ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, OperationCreates, OperationReads,
    OperationWrites,
};

use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial,
    ApplicationOperationBodyBuilder, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreation,
};

pub struct ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity> {
    parent: ApplicationOperationBodyBuilder<Schema, Operation, Input>,
    entity: String,
    key: String,
    fields: Vec<ApplicationOperationBodyCreatedField>,
    marker: PhantomData<fn() -> Entity>,
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Creates in the admitted existing scope's context. The key is a
    /// candidate-local symbolic slot, not a business or idempotency identity.
    pub fn create_in_scope<Entity>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: &str,
    ) -> Result<
        ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>,
        ApplicationOperationBodyAuthoringDenial,
    >
    where
        Entity: OperationCreates<Operation>,
    {
        if key.is_empty() || key.trim() != key {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidCreationKey);
        }
        Ok(ApplicationOperationBodyCreationBuilder {
            parent: self,
            entity: entity.name().to_owned(),
            key: key.to_owned(),
            fields: Vec::new(),
            marker: PhantomData,
        })
    }
}

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    pub fn initialize<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        Field::Binding::validate(&value)
            .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
        self.fields
            .push(ApplicationOperationBodyCreatedField::Literal {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                value: Field::Binding::encode(&value)?,
            });
        Ok(self)
    }

    pub fn initialize_input<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.fields
            .push(ApplicationOperationBodyCreatedField::Input {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                member: member.to_owned(),
            });
        Ok(self)
    }

    /// Copies the scope's tracked preimage into one created field. Both fields
    /// share the same value, unit and exact codec; the host supplies no value.
    pub fn initialize_from_scope<
        SourceEntity,
        SourceAspect,
        SourceField,
        SourceWrite,
        SourceEquality,
        TargetAspect,
        TargetField,
        TargetWrite,
        TargetEquality,
        Value,
        Unit,
    >(
        mut self,
        source: ApplicationFieldRef<
            Schema,
            SourceEntity,
            SourceAspect,
            SourceField,
            Value,
            SourceWrite,
            SourceEquality,
            Unit,
        >,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            TargetAspect,
            TargetField,
            Value,
            TargetWrite,
            TargetEquality,
            Unit,
        >,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        SourceField: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        TargetField: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if SourceField::Binding::IDENTITY != TargetField::Binding::IDENTITY
            || SourceField::Binding::SCALAR_FAMILY != TargetField::Binding::SCALAR_FAMILY
        {
            return Err(ApplicationOperationBodyAuthoringDenial::IncompatibleSourceBinding);
        }
        self.fields
            .push(ApplicationOperationBodyCreatedField::ScopeField {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: TargetField::Binding::IDENTITY,
                scalar_family: TargetField::Binding::SCALAR_FAMILY,
                source: ApplicationFieldBindingLocus::new(
                    source.entity(),
                    source.aspect(),
                    source.field(),
                ),
            });
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<
        ApplicationOperationBody<Schema, Operation, Input>,
        ApplicationOperationBodyAuthoringDenial,
    > {
        let mut body = self.parent.finish()?;
        let scope = body
            .program
            .scope_entity()
            .expect("completed body has a scope");
        let mut fields = std::collections::BTreeSet::new();
        for step in &self.fields {
            if !fields.insert(step.field()) {
                return Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
            }
            if step
                .source_field()
                .is_some_and(|source| source.entity() != scope)
            {
                return Err(ApplicationOperationBodyAuthoringDenial::ForeignSourceEntity);
            }
        }
        body.program.creation = Some(ApplicationOperationBodyCreation {
            entity: self.entity,
            key: self.key,
            fields: self.fields,
        });
        Ok(body)
    }
}
