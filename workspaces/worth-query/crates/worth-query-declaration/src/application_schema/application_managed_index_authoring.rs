use super::{ApplicationSchema, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use crate::application_query::{
    ApplicationManagedIndexBinding, ApplicationManagedIndexBindingDescriptor,
    ApplicationManagedIndexDefinition,
};

impl<S: ApplicationSchema> ApplicationSchemaDeclarationBuilder<S> {
    /// Declares provider meaning and retains the actual typed source/probe associations.
    pub fn managed_index<I: ApplicationManagedIndexBinding<S>>(
        mut self,
        definition: ApplicationManagedIndexDefinition<S, I>,
    ) -> Self {
        let descriptor = ApplicationManagedIndexBindingDescriptor::of::<S, I>();
        if !self
            .member_provenance
            .query_bindings()
            .contains(descriptor.source())
        {
            self.member_provenance
                .register_query_binding(descriptor.source().clone());
        }
        self.member_provenance.register_managed_index(descriptor);
        self.push_member(ApplicationSchemaMember::ManagedIndex {
            definition: definition.into_erased(),
        })
    }
}
