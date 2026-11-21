//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn validate_installed_entry_for_test(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        Self::validate_installed_entry(self, runtime, binding, installed)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn validate_installed_operation_for_test(
        &self,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        Self::validate_installed_operation(self, installed)
    }
}
