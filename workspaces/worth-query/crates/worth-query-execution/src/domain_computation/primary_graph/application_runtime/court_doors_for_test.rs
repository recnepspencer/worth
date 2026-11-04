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
    pub fn program_support_for_test(
        &self,
    ) -> &Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        &self.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_support_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>
    {
        &mut self.program_support
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn installed_program_support_for_test(
        &self,
    ) -> Option<&super::super::program_occurrence::WorthQueryInstalledProgramSupport<Schema>> {
        Self::installed_program_support(self)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn mutation_projection_for_test(
        &self,
    ) -> &super::super::WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        &self.mutation_projection
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_required_operations_for_test(&self) -> &std::collections::HashSet<TypeId> {
        &self.program_required_operations
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    #[doc(hidden)]
    pub fn graph_work_resource_support_ref_for_test(
    &self) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot{
        Self::graph_work_resource_support_ref(self)
    }
}

