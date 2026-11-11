//! Typed operation membership is checked before selectors are erased.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_schema::{ApplicationOperationRef, ApplicationRelationRef, OperationReads};
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
    pub(in crate::application_schema::operation_definition::body) fn into_operands(
        self,
    ) -> Vec<Operand> {
        self.operands
    }
    /// Uses the value binding already selected from a fragment's record.
    pub fn record_field<E, A, T>(
        self,
        name: &str,
        field: crate::application_schema::fragment::RecordField<E, A, T>,
    ) -> Result<Self, Denial> {
        if field.recipe().unit().is_some() {
            return Err(Denial::UnsupportedScalar);
        }
        self.push_record(name, field, false)
    }
    pub fn record_presence<E, A, T>(
        self,
        name: &str,
        field: crate::application_schema::fragment::RecordField<E, A, T>,
    ) -> Result<Self, Denial> {
        self.push_record(name, field, true)
    }
    fn push_record<E, A, T>(
        self,
        name: &str,
        field: crate::application_schema::fragment::RecordField<E, A, T>,
        presence_only: bool,
    ) -> Result<Self, Denial> {
        self.push(
            name,
            Source::Field {
                field: field.locus().clone(),
                binding: field.recipe().binding_identity().clone(),
                scalar_family: field.recipe().scalar_family(),
                presence: field.presence(),
                presence_only,
            },
        )
    }

    /// Reads a scalar value with its exact optionality. Unit-bearing values
    /// need a quantity binding rather than silently becoming unitless scalars.
    pub fn field<Field: crate::application_schema::fragment::RecordSelection>(
        self,
        name: &str,
        field: Field,
    ) -> Result<Self, Denial> {
        let checked = field.checked().map_err(|_| Denial::InvalidRecordField {
            field: field.expected_recipe().locus().clone(),
        })?;
        self.record_field(name, checked)
    }

    /// Reads presence without interpreting the field's value or physical unit.
    pub fn presence<Field: crate::application_schema::fragment::RecordSelection>(
        self,
        name: &str,
        field: Field,
    ) -> Result<Self, Denial> {
        let checked = field.checked().map_err(|_| Denial::InvalidRecordField {
            field: field.expected_recipe().locus().clone(),
        })?;
        self.record_presence(name, checked)
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

    pub(super) fn push(mut self, name: &str, source: Source) -> Result<Self, Denial> {
        source.expression_type()?;
        if self.operands.iter().any(|operand| operand.name() == name) {
            return Err(Denial::DuplicateOperand);
        }
        self.operands
            .push(Operand::from_untrusted(name.to_owned(), source));
        Ok(self)
    }
}
