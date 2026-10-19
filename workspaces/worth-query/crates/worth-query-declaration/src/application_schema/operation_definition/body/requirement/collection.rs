//! A collection selector binds its child field before erasure, not a caller-supplied list.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationRelationRef,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, NoApplicationUnit,
    OperationReads,
};

impl<Schema, Operation> ApplicationOperationBodyRequirementOperands<Schema, Operation> {
    /// Observe one field of every distinct live outgoing endpoint. Empty membership
    /// is an empty list; optional absence stays an Option element. Foundation's
    /// normal list expressions own cardinality and per-member satisfaction.
    pub fn outgoing_field_values<Relation, From, To, Aspect, Field, Value, Write, Equality>(
        self,
        name: &str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        field: ApplicationFieldRef<
            Schema,
            To,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            NoApplicationUnit,
        >,
    ) -> Result<Self, Denial>
    where
        Relation: OperationReads<Operation>,
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
    {
        self.push(
            name,
            Source::RelatedFieldValues {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                presence: Field::PRESENCE,
            },
        )
    }
}
