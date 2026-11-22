//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn belongs_to_for_test(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        Self::belongs_to(self, runtime_authority, binding_identity)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn validate_installed_mutation_binding_for_test(
        &self,
        descriptor: Option<&ApplicationMutationBindingDescriptor>,
    ) -> Result<(), MutationHandlerExecutionDenial> {
        Self::validate_installed_mutation_binding(self, descriptor)
    }
}
