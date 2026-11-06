//! A collection selector binds its child field before erasure, not a caller-supplied list.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_schema::{
    ApplicationRelationRef, ApplicationScalarValueBinding, OperationReads,
};

impl<Schema, Operation> ApplicationOperationBodyRequirementOperands<Schema, Operation> {
    /// Observe one field of every distinct live outgoing endpoint. Empty membership
    /// is an empty list; optional absence stays an Option element. Foundation's
    /// normal list expressions own cardinality and per-member satisfaction.
    pub fn outgoing_field_values<Relation, From, To, Field>(
        self,
        name: &str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        field: Field,
    ) -> Result<Self, Denial>
    where
        Relation: OperationReads<Operation>,
        Field: crate::application_schema::fragment::RecordSelection<Entity = To>,
    {
        let field = field.checked().map_err(|_| Denial::InvalidRecordField {
            field: field.expected_recipe().locus().clone(),
        })?;
        self.push(
            name,
            Source::RelatedFieldValues {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                field: field.locus().clone(),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                presence: field.presence(),
            },
        )
    }
}
