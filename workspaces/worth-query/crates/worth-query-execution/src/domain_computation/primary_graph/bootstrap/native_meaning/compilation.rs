//! One compiler produces the complete Native layout, rules and Source owners.
use super::*;
use worth_relational::facade::{
    identity::KindId, runtime::CustomInvariantRegistration, schema::RelationalSchemaRegistry,
};

pub(in crate::domain_computation::primary_graph) enum NativeSchemaCompilation<'source> {
    Fresh(&'source RelationalSchemaRegistry),
    Original {
        base: &'source RelationalSchemaRegistry,
        source: &'source RelationalSchemaRegistry,
        maximum_kind: Option<KindId>,
        maximum_aspect: Option<worth_foundational::facade::AspectIdentity>,
    },
    Successor {
        base: &'source RelationalSchemaRegistry,
        predecessor: &'source WorthQueryPrimaryGraphLayout,
    },
}

pub(in crate::domain_computation::primary_graph) struct CompiledNativeSchemaMeaning {
    pub(in crate::domain_computation::primary_graph) layout: WorthQueryPrimaryGraphLayout,
    pub(in crate::domain_computation::primary_graph) additions: RelationalSchemaRegistry,
    pub(in crate::domain_computation::primary_graph) registrations:
        Vec<CustomInvariantRegistration>,
    pub(in crate::domain_computation::primary_graph) managed_sources:
        super::super::super::application_query::managed_index::ManagedSourceInstallation,
    pub(in crate::domain_computation::primary_graph) source_initializer:
        Option<super::super::super::application_query::managed_index::SourceInitializationIssuer>,
}
impl WorthQueryExecutionInstallationAuthority {
    pub(in crate::domain_computation::primary_graph) fn compile_native_schema<
        Schema: ApplicationSchema,
    >(
        &self,
        phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        source: NativeSchemaCompilation<'_>,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<CompiledNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
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
        let schema = installed_schema.installed_declaration();
        let contracts = installed_schema.native_contracts();
        let values = installed_schema.value_bindings();
        let (layout, mut additions) = match source {
            NativeSchemaCompilation::Fresh(base) => {
                WorthQueryPrimaryGraphLayout::lower(schema, contracts, base, values)
            }
            NativeSchemaCompilation::Original {
                base,
                source,
                maximum_kind,
                maximum_aspect,
            } => WorthQueryPrimaryGraphLayout::lower_with_original(
                schema,
                contracts,
                base,
                values,
                source,
                maximum_kind,
                maximum_aspect,
            ),
            NativeSchemaCompilation::Successor { base, predecessor } => {
                WorthQueryPrimaryGraphLayout::lower_successor(
                    schema,
                    contracts,
                    base,
                    values,
                    predecessor,
                )
            }
        }?;
        if invariant_factories.current_publication_ownership() {
            additions = super::super::super::schema_layout::install_current_publication_ownership(
                additions,
            )?;
        }
        let (registrations, managed_sources, source_initializer) = invariant_factories.lower(
            &phase.execution_request(),
            installed_schema.binding_identity(),
            &layout,
        )?;
        Ok(CompiledNativeSchemaMeaning {
            layout,
            additions,
            registrations,
            managed_sources,
            source_initializer,
        })
    }
}
