//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryApplicationInvariantFactories<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn lower_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: &crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout,
    ) -> Result<
        (
            Vec<CustomInvariantRegistration>,
            super::super::application_query::managed_index::ManagedSourceInstallation,
            Option<super::super::application_query::managed_index::SourceInitializationIssuer>,
        ),
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        Self::lower(self, execution, binding_identity, layout)
    }
}
