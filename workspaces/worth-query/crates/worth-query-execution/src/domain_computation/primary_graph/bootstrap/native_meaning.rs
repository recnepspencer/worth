//! Exclusive schema and custom-invariant installation on the actual Native owner.
use super::*;
use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;

pub(in crate::domain_computation::primary_graph) struct InstalledNativeSchemaMeaning {
    pub(in crate::domain_computation::primary_graph) managed_views:
        Arc<super::super::application_query::derived_view::ManagedDerivedViewRegistry>,
    pub(in crate::domain_computation::primary_graph) source_initializer:
        Option<super::super::application_query::managed_index::SourceInitializationIssuer>,
    pub(in crate::domain_computation::primary_graph) layout: WorthQueryPrimaryGraphLayout,
    pub(in crate::domain_computation::primary_graph) receipt:
        RelationalInitialSchemaInstallationReceipt,
    pub(in crate::domain_computation::primary_graph) expected_invariant_inventory_digest: [u8; 32],
}

impl WorthQueryExecutionInstallationAuthority {
    pub(in crate::domain_computation::primary_graph) fn prepare_native_schema<
        Schema: ApplicationSchema,
    >(
        &self,
        _phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: &mut RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<InstalledNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        self.prepare_native_schema_with_initialization(
            _phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_native_schema_for_fresh_home<
        Schema: ApplicationSchema,
    >(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: &mut RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        initialization: worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
    ) -> Result<InstalledNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        self.prepare_native_schema_with_initialization(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            Some(initialization),
        )
    }

    fn prepare_native_schema_with_initialization<Schema: ApplicationSchema>(
        &self,
        _phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: &mut RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        initialization: Option<
            worth_proof::FreshNativeInitialization<
                worth_relational::facade::durability::RelationalFreshInitialization,
            >,
        >,
    ) -> Result<InstalledNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        if !self.belongs_to(runtime) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime,
                "execution installation authority belongs to another runtime",
            ));
        }
        if runtime.primary_graph().is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::AlreadyInstalled,
                "execution runtime already owns a primary graph",
            ));
        }
        runtime
            .installed_packages()
            .validate_application_schema(installed_schema)
            .map_err(|denial| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                    denial.subject(),
                )
            })?;
        let (layout, additions) = WorthQueryPrimaryGraphLayout::lower(
            installed_schema.installed_declaration(),
            installed_schema.native_contracts(),
            &relational_runtime.config().schema.registry,
            installed_schema.value_bindings(),
        )?;
        let (registrations, managed_sources, source_initializer) = invariant_factories.lower(
            &_phase.execution_request(),
            installed_schema.binding_identity(),
            &layout,
        )?;
        let (invariant_installation_receipt, expected_invariant_inventory_digest) =
            super::initial_installation::install(
                relational_runtime,
                additions,
                registrations,
                &layout,
                &_phase.execution_request(),
                initialization,
            )?;
        let managed_views = Arc::new(
            super::super::application_query::derived_view::ManagedDerivedViewRegistry::with_sources(
                managed_sources,
                &relational_runtime.main_branch_identity(),
            ),
        );
        Ok(InstalledNativeSchemaMeaning {
            managed_views,
            source_initializer,
            layout,
            receipt: invariant_installation_receipt,
            expected_invariant_inventory_digest,
        })
    }
}
