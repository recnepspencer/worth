//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_schema_for_test(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.installed_schema
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn primary_provider_for_test(&self) -> &std::sync::Arc<WorthQueryPrimaryGraphProvider> {
        &self.primary_provider
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn product_runtime_for_test(&self) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime {
        &self.product_runtime
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_support_for_test(&self) -> &Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        &self.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> &WorthQueryExecutionRuntime {
        &self.runtime
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_support_mut_for_test(&mut self) -> &mut Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        &mut self.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_program_support_for_test(
        &self) -> Option<&super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        Self::installed_program_support(self)
    }
}
