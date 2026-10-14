//! Body authoring derives each target's binding from its typed field.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    OperationReads, OperationWrites, OptionalApplicationFieldValue, WritableCapability,
};

use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial,
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};

pub struct ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    steps: Vec<ApplicationOperationBodyScopeField>,
    marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    /// Begins scope-bound source expectations and assignments. Membership is
    /// derived when the completed body is attached to this operation definition.
    pub fn body(self) -> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
        ApplicationOperationBodyBuilder {
            steps: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    pub fn expect<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        Field::Binding::validate(&value)
            .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
        self.steps.push(ApplicationOperationBodyScopeField::Expect {
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            binding: Field::Binding::IDENTITY,
            value: Field::Binding::encode(&value)?,
        });
        Ok(self)
    }

    pub fn assign<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        Field::Binding::validate(&value)
            .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
        self.steps.push(ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            binding: Field::Binding::IDENTITY,
            value: Field::Binding::encode(&value)?,
        });
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<
        ApplicationOperationBody<Schema, Operation, Input>,
        ApplicationOperationBodyAuthoringDenial,
    > {
        let Some(first) = self.steps.first() else {
            return Err(ApplicationOperationBodyAuthoringDenial::Empty);
        };
        let mut writes = BTreeSet::new();
        for step in &self.steps {
            if step.field().entity() != first.field().entity() {
                return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
            }
            if step.write_target().is_some() && !writes.insert(step.field().clone()) {
                return Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
            }
        }
        if writes.is_empty() {
            return Err(ApplicationOperationBodyAuthoringDenial::Empty);
        }
        Ok(ApplicationOperationBody {
            program: ApplicationOperationBodyProgram {
                scope_fields: self.steps,
                creation: None,
            },
            marker: PhantomData,
        })
    }

    /// Assigns a scalar from the original canonical input's serialized member.
    /// Selection and destination validation are native; no getter is installed.
    pub fn assign_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::AssignInput {
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

    /// Sets one optional scalar from a flat member of the canonical input.
    /// The member must be present, including when its value is empty or zero.
    pub fn set_optional_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::SetOptionalInput {
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

    /// Clears one optional scalar; absence is not an empty value.
    pub fn clear_optional<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        self.steps
            .push(ApplicationOperationBodyScopeField::ClearOptional {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
            });
        self
    }
}
