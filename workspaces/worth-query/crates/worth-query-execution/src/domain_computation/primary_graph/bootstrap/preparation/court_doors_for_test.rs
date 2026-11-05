//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExecutionInstallationAuthority {
    #[doc(hidden)]
    pub fn prepare_primary_graph_with_relational_runtime_for_test<Schema>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        Self::prepare_primary_graph_with_relational_runtime::<Schema>(
            self,
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
        )
    }
}

impl WorthQueryExecutionInstallationAuthority {
    #[doc(hidden)]
    pub fn prepare_primary_graph_with_relational_runtime_and_invariants_for_test<Schema>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        Self::prepare_primary_graph_with_relational_runtime_and_invariants::<Schema>(
            self,
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
        )
    }
}
