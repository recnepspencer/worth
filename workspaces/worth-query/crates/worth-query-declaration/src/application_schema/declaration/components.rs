//! Add one immutable, normalized set to the ordinary schema declaration.
use crate::application_schema::{
    ApplicationComponentSetDefinition, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
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
