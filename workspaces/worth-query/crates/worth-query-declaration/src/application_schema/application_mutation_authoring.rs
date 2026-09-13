use crate::application_operation::ApplicationMutationBinding;

use super::ApplicationSchemaDeclarationBuilder;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Registers one complete typed mutation binding for installation.
    pub fn application_mutation_binding<Binding>(mut self) -> Self
    where
        Schema: super::ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let descriptor = Binding::descriptor();
        self.push_member_in_place(super::ApplicationSchemaMember::ApplicationMutation {
            description: descriptor.description().clone(),
        });
        self.member_provenance.register_mutation_binding(descriptor);
        self
    }
}
