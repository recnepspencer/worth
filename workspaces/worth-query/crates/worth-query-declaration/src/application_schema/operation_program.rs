use super::capabilities::{
    ApplicationFieldUnit, OperationCreates, OperationDeletes, OperationEmits, OperationLinks,
    OperationUnlinks, OperationWrites,
};
use super::field_reference::ApplicationFieldRef;
use super::references::{
    ApplicationEffectRef, ApplicationEntityRef, ApplicationOperationRef, ApplicationRelationRef,
};
use super::{
    ApplicationOperationProgramTarget, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaMember, DeclaredApplicationFieldValue,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_create<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationCreates<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Create {
                entity: entity.name().to_string(),
            },
        )
    }

    pub fn operation_delete<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationDeletes<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Delete {
                entity: entity.name().to_string(),
            },
        )
    }

    pub fn operation_write<Operation, Input, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: OperationWrites<Operation>,
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Write {
                entity: field.entity().to_string(),
                aspect: field.aspect().to_string(),
                field: field.field().to_string(),
            },
        )
    }

    pub fn operation_link<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationLinks<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Link {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    pub fn operation_unlink<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationUnlinks<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Unlink {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    pub fn operation_emit<Operation, Input, Effect, Payload>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
    ) -> Self
    where
        Effect: OperationEmits<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Emit {
                effect: effect.name().to_string(),
            },
        )
    }

    fn program(self, operation: &str, target: ApplicationOperationProgramTarget) -> Self {
        self.push_member(ApplicationSchemaMember::OperationProgram {
            operation: operation.to_string(),
            target,
        })
    }
}
