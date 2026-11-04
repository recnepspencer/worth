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
    pub fn product_runtime_for_test(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.product_runtime
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
    pub fn runtime_for_test(&self) -> &WorthQueryExecutionRuntime {
        &self.runtime
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
    pub fn bridge_for_test(
        &self,
    ) -> &super::super::managed_bridge::WorthQueryInstalledApplicationBridge {
        &self.bridge
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
    pub fn authorization_for_test(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry> {
        &self.authorization
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn output_demands_for_test(
        &self,
    ) -> &super::super::application_output_demand::WorthQueryOutputDemandRegistry {
        &self.output_demands
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn output_producer_routes_for_test(
        &self,
    ) -> &super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes {
        &self.output_producer_routes
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn primary_graph_authority_for_test(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority {
        &self.primary_graph_authority
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn program_required_operations_for_test(&self) -> &std::collections::HashSet<TypeId> {
        &self.program_required_operations
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn relational_branch_identity_for_test(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity {
        &self.relational_branch_identity
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn fix_authentication_time_for_test(&mut self, now: std::time::Instant) {
        Self::fix_authentication_time(self, now)
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
