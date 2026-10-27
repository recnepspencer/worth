use crate::application_resource::{
    ApplicationResourceBinding, ApplicationResourceBindingDescriptor, ApplicationResourceDefinition,
};

use super::{ApplicationSchema, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema: ApplicationSchema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Register actual compiler-local resource recipes alongside portable meaning.
    pub fn application_resource<Binding: ApplicationResourceBinding<Schema>>(
        mut self,
        definition: ApplicationResourceDefinition<Schema, Binding>,
    ) -> Self {
        let descriptor = ApplicationResourceBindingDescriptor::of::<Schema, Binding>();
        if !self
            .member_provenance
            .query_bindings()
            .contains(descriptor.access())
        {
            self.member_provenance
                .register_query_binding(descriptor.access().clone());
        }
        self.member_provenance.register_resource(descriptor);
        self.push_member(ApplicationSchemaMember::ApplicationResource {
            definition: definition.into_erased(),
        })
    }
}
