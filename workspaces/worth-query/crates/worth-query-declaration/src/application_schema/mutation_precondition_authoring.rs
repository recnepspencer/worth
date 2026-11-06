//! Preconditions select the same typed record columns as ordinary operations.
use super::fragment::RecordSelection;
use super::{
    ApplicationMutationPreconditionFamily, ApplicationMutationPreconditionTarget,
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_expected_version<Operation, Input, Field: RecordSelection>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: Field,
    ) -> Self {
        self.precondition(
            operation.name(),
            ApplicationMutationPreconditionFamily::ExpectedVersion,
            field,
        )
    }
    pub fn operation_expected_fact<Operation, Input, Field: RecordSelection>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: Field,
    ) -> Self {
        self.precondition(
            operation.name(),
            ApplicationMutationPreconditionFamily::ExpectedFact,
            field,
        )
    }
    fn precondition<Field: RecordSelection>(
        mut self,
        operation: &str,
        family: ApplicationMutationPreconditionFamily,
        field: Field,
    ) -> Self {
        let recipe = field.expected_recipe();
        let locus = recipe.locus();
        let target = ApplicationMutationPreconditionTarget::field(
            family,
            locus.entity(),
            locus.aspect(),
            locus.field(),
        );
        self.member_provenance.fragments.expect_field(recipe);
        self.push_member(ApplicationSchemaMember::OperationMutationPrecondition {
            operation: operation.to_owned(),
            target,
        })
    }
}
