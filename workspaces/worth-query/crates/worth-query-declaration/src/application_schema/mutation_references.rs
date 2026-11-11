//! Referenced actions require an independently declared typed operation and binding.
use crate::application_operation::{
    ApplicationMutationBinding, ApplicationMutationBindingDescriptor,
};

use super::{
    ApplicationSchema, ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
    ApplicationSchemaMemberProvenance,
};

#[derive(Clone, Debug, Default)]
pub(super) struct MutationReferences(Vec<ApplicationMutationBindingDescriptor>);

impl MutationReferences {
    pub(super) fn validate(
        &self,
        provenance: &ApplicationSchemaMemberProvenance,
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        // Member identity uniqueness runs first: presence here therefore proves one owner.
        for expected in &self.0 {
            if !provenance.admits_mutation_binding_operation(expected)
                || !provenance.mutation_bindings().contains(expected)
            {
                return Err(
                    ApplicationSchemaDeclarationDenial::MissingReferencedMutation {
                        operation: expected.operation_name().to_owned(),
                        binding: expected.identity().to_owned(),
                    },
                );
            }
        }
        Ok(())
    }
}

impl<Schema: ApplicationSchema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Records an action reference without declaring its operation or mutation binding.
    /// Schema composition requires an owner of both typed declarations.
    #[doc(hidden)]
    pub fn reference_application_mutation_binding<Binding>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.mutation_references.0.push(Binding::descriptor());
        self
    }
}
