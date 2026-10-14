//! Literal authoring uses the field's declared value binding, never raw values.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    OperationReads, OperationWrites, WritableCapability,
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
            if let ApplicationOperationBodyScopeField::Assign { field, .. } = step {
                if !writes.insert(field.clone()) {
                    return Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
                }
            }
        }
        if writes.is_empty() {
            return Err(ApplicationOperationBodyAuthoringDenial::Empty);
        }
        Ok(ApplicationOperationBody {
            program: ApplicationOperationBodyProgram {
                scope_fields: self.steps,
            },
            marker: PhantomData,
        })
    }
}
