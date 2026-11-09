use super::*;
use crate::application_schema::fragment::RelationSelection;
impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Authorizes this record field only while the operation creates its relation.
    pub fn operation_initialize_relation_field<Operation, Input, Field>(
        mut self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: Field,
    ) -> Self
    where
        Field: RelationSelection,
        Field::Entity: OperationLinks<Operation>,
    {
        self.member_provenance
            .fragments
            .expect_field(field.expected_recipe());
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::InitializeRelationField {
                relation: field.entity().to_owned(),
                from: field.from().to_owned(),
                to: field.to().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            },
        )
    }
}
