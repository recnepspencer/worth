use super::capabilities::{ApplicationFieldUnit, OperationReads};
use super::field_reference::ApplicationFieldRef;
use super::references::{ApplicationEntityRef, ApplicationOperationRef, ApplicationRelationRef};
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaMember, DeclaredApplicationFieldValue,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_decision_fact_budget<Operation, Input>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        maximum_fact_count: usize,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::OperationDecisionFactBudget {
            operation: operation.name().to_string(),
            maximum_fact_count,
        })
    }

    pub fn operation_projection_work_budget<Operation, Input>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        maximum_work_units: usize,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: operation.name().to_string(),
            maximum_work_units,
        })
    }

    pub fn operation_read_entity<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationReads<Operation>,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::Entity {
                entity: entity.name().to_string(),
            },
        )
    }

    pub fn operation_read_field<
        Operation,
        Input,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: OperationReads<Operation>,
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::Field {
                entity: field.entity().to_string(),
                aspect: field.aspect().to_string(),
                field: field.field().to_string(),
            },
        )
    }

    pub fn operation_read_relation<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationReads<Operation>,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::Relation {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    fn decision_read(
        self,
        operation: &str,
        target: ApplicationOperationDecisionReadTarget,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::OperationDecisionRead {
            operation: operation.to_string(),
            target,
        })
    }
}
