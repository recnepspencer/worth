//! An optional source must actually be present; no sentinel or missing-value default is copied.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldPresence, ApplicationFieldRef,
    ApplicationFieldUnit, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    OperationReads, OperationWrites, OptionalApplicationFieldValue,
};

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    pub fn initialize_from_present_scope<
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
        target: ApplicationFieldRef<
            Schema,
            Entity,
            TargetAspect,
            TargetField,
            Value,
            TargetWrite,
            TargetEquality,
            Unit,
        >,
    ) -> Result<Self, Denial>
    where
        SourceField: OperationReads<Operation> + OptionalApplicationFieldValue<Value = Value>,
        TargetField: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if TargetField::PRESENCE != ApplicationFieldPresence::Required
            || SourceField::Binding::IDENTITY != TargetField::Binding::IDENTITY
            || SourceField::Binding::SCALAR_FAMILY != TargetField::Binding::SCALAR_FAMILY
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Step::PresentScopeField {
            field: ApplicationFieldBindingLocus::new(
                target.entity(),
                target.aspect(),
                target.field(),
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
}
