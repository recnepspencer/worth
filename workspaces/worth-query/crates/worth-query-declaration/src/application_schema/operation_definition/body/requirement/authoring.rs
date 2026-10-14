//! Typed operation membership is checked before selectors are erased.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationRef, ApplicationRelationRef, ApplicationScalarValueBinding,
    DeclaredApplicationFieldValue, NoApplicationUnit, OperationReads,
};
use std::marker::PhantomData;

/// Typed native sources available to one operation's requirement expressions.
pub struct ApplicationOperationBodyRequirementOperands<Schema, Operation> {
    pub(super) operands: Vec<Operand>,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    pub fn requirement_operands(
        self,
    ) -> ApplicationOperationBodyRequirementOperands<Schema, Operation> {
        ApplicationOperationBodyRequirementOperands {
            operands: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation> ApplicationOperationBodyRequirementOperands<Schema, Operation> {
    /// Reads a scalar value with its exact optionality. Unit-bearing values
    /// need a quantity binding rather than silently becoming unitless scalars.
    pub fn field<Entity, Aspect, Field, Value, Write, Equality>(
        self,
        name: &str,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            NoApplicationUnit,
        >,
    ) -> Result<Self, Denial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
    {
        self.add_field::<Entity, Aspect, Field, Value, Write, Equality, NoApplicationUnit>(
            name, field, false,
        )
    }

    /// Reads presence without interpreting the field's value or physical unit.
    pub fn presence<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        self,
        name: &str,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Self, Denial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.add_field(name, field, true)
    }

    fn add_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        self,
        name: &str,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        presence_only: bool,
    ) -> Result<Self, Denial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.push(
            name,
            Source::Field {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                presence: Field::PRESENCE,
                presence_only,
            },
        )
    }

    /// Counts distinct live target entities. A dangling endpoint is unavailable,
    /// and duplicate edges to one entity do not inflate the count.
    pub fn outgoing_item_count<Relation, From, To>(
        self,
        name: &str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Result<Self, Denial>
    where
        Relation: OperationReads<Operation>,
    {
        self.related(name, relation, Direction::Outgoing)
    }

    /// Counts distinct live source entities of incoming relations.
    pub fn incoming_item_count<Relation, From, To>(
        self,
        name: &str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Result<Self, Denial>
    where
        Relation: OperationReads<Operation>,
    {
        self.related(name, relation, Direction::Incoming)
    }

    fn related<Relation, From, To>(
        self,
        name: &str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        direction: Direction,
    ) -> Result<Self, Denial> {
        self.push(
            name,
            Source::RelatedItemCount {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                direction,
            },
        )
    }

    fn push(mut self, name: &str, source: Source) -> Result<Self, Denial> {
        source.expression_type()?;
        if self.operands.iter().any(|operand| operand.name() == name) {
            return Err(Denial::DuplicateOperand);
        }
        self.operands
            .push(Operand::from_untrusted(name.to_owned(), source));
        Ok(self)
    }
}
