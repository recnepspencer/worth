//! Exclusive schema and custom-invariant installation on the actual Native owner.
use super::*;
use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;
mod compilation;
mod inventory;
pub(in crate::domain_computation::primary_graph) use compilation::{
    CompiledNativeSchemaMeaning, NativeSchemaCompilation,
};
pub(in crate::domain_computation::primary_graph) use inventory::NativeSchemaInventory;

pub(in crate::domain_computation::primary_graph) struct PreparedNativeSchemaMeaning {
    pub(in crate::domain_computation::primary_graph) managed_sources:
        super::super::application_query::managed_index::ManagedSourceInstallation,
    pub(in crate::domain_computation::primary_graph) source_initializer:
        Option<super::super::application_query::managed_index::SourceInitializationIssuer>,
    pub(in crate::domain_computation::primary_graph) layout: WorthQueryPrimaryGraphLayout,
    pub(in crate::domain_computation::primary_graph) receipt:
        RelationalInitialSchemaInstallationReceipt,
    pub(in crate::domain_computation::primary_graph) expected_invariant_inventory_digest: [u8; 32],
}

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
    ) -> Result<PreparedNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        self.prepare_native_schema_with_initialization(
            _phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            None,
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
    ) -> Result<PreparedNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        self.prepare_native_schema_with_initialization(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            Some(initialization),
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_native_schema_from_original<
        Schema: ApplicationSchema,
    >(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: &mut RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        source: &worth_relational::facade::schema::RelationalSchemaRegistry,
        maximum_kind: Option<worth_relational::facade::identity::KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
    ) -> Result<PreparedNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        self.prepare_native_schema_with_initialization(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            invariant_factories,
            None,
            Some((source, maximum_kind, maximum_aspect)),
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
        source: Option<(
            &worth_relational::facade::schema::RelationalSchemaRegistry,
            Option<worth_relational::facade::identity::KindId>,
            Option<worth_foundational::facade::AspectIdentity>,
        )>,
    ) -> Result<PreparedNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        let config = relational_runtime.config();
        let source = match source {
            Some((source, maximum_kind, maximum_aspect)) => NativeSchemaCompilation::Original {
                base: &config.schema.registry,
                source,
                maximum_kind,
                maximum_aspect,
            },
            None => NativeSchemaCompilation::Fresh(&config.schema.registry),
        };
        let compiled = self.compile_native_schema(
            _phase,
            runtime,
            installed_schema,
            source,
            invariant_factories,
        )?;
        super::initial_installation::install(
            relational_runtime,
            compiled,
            &_phase.execution_request(),
            initialization,
        )
    }
}

impl PreparedNativeSchemaMeaning {
    pub(in crate::domain_computation::primary_graph) fn bind_sources(
        self,
        owner: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> InstalledNativeSchemaMeaning {
        InstalledNativeSchemaMeaning {
            managed_views: Arc::new(
                super::super::application_query::derived_view::ManagedDerivedViewRegistry::with_sources(
                    self.managed_sources, owner,
                ),
            ),
            source_initializer: self.source_initializer,
            layout: self.layout,
            receipt: self.receipt,
            expected_invariant_inventory_digest: self.expected_invariant_inventory_digest,
        }
    }
}
