use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductWorldResources, WorthQueryExecutionInstallationAuthority,
    WorthQueryExecutionRuntime,
};

use super::{
    WorthQueryApplicationInvariantFactories, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};

impl WorthQueryExecutionInstallationAuthority {
    /// Prepares the application's in-memory graph with its exact invariant factories.
    pub fn prepare_primary_graph_with_invariants<Schema: ApplicationSchema>(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    {
        use worth_relational::facade::runtime::RelationalRuntimeApi;

        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            runtime,
            installed_schema,
            RelationalRuntimeApi::builder().build(),
            product_world_resources,
            invariant_factories,
        )
    }
}
