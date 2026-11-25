use super::*;

impl WorthQueryExecutionInstallationAuthority {
    pub fn prepare_primary_graph<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            phase,
            runtime,
            installed_schema,
            RelationalRuntimeApi::builder().build()?,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        self.prepare_native_graph(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
        )
        .and_then(|prepared| {
            prepared.into_bootstrap(runtime, installed_schema, product_world_resources)
        })
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_native_graph<
        Schema: ApplicationSchema,
    >(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        mut relational_runtime: RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<
        super::prepared_native::PreparedPrimaryGraphInstallation,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        let schema = self.prepare_native_schema(
            phase,
            runtime,
            installed_schema,
            &mut relational_runtime,
            invariant_factories,
        )?;
        let super::native_meaning::InstalledNativeSchemaMeaning {
            managed_views,
            source_initializer,
            layout,
            receipt: invariant_installation_receipt,
            expected_invariant_inventory_digest,
        } = schema.bind_sources(&relational_runtime.main_branch_identity());
        let mut graph = super::prepared_native::PreparedNativeGraph::new(
            relational_runtime,
            layout,
            installed_schema.binding_identity(),
            managed_views,
            false,
        )?;
        graph.bind_application_publication(&invariant_installation_receipt);
        Ok(super::prepared_native::PreparedPrimaryGraphInstallation {
            graph,
            meaning: super::prepared_native::PreparedPrimaryGraphMeaning {
                resource_support:
                    super::super::provider::WorthQueryPrimaryGraphResourceSupport::install(
                        runtime
                            .application_query_resource_profile()
                            .maximum_concurrent_graph_work(),
                    ),
                recovered_publication: None,
                recovered_relational_authority: None,
                recovered_checkpoint_restore_work: None,
                invariant_installation_receipt,
                expected_invariant_inventory_digest,
                typed_rows: Default::default(),
                source_initializer,
            },
        })
    }
}

mod checkpoint_recovery_denial;
pub(in crate::domain_computation::primary_graph) use checkpoint_recovery_denial::map_checkpoint_recovery_stop;

mod court_doors_for_test;
