//! Add one immutable, normalized set to the ordinary schema declaration.
use crate::application_schema::{
    ApplicationComponentSetDefinition, ApplicationOperationComponentReadDefinition,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_component_read<Operation, Root, Set, Member>(
        mut self,
        definition: ApplicationOperationComponentReadDefinition<
            Schema,
            Operation,
            Root,
            Set,
            Member,
        >,
    ) -> Self {
        self.members
            .push(ApplicationSchemaMember::OperationComponentRead {
                definition: definition.into_meaning(),
            });
        self
    }

    pub fn components<Set, Entity>(
        mut self,
        definition: ApplicationComponentSetDefinition<Schema, Set, Entity>,
    ) -> Self {
        self.members.push(ApplicationSchemaMember::ComponentSet {
            definition: definition.into_meaning(),
        });
        self
    }
}
