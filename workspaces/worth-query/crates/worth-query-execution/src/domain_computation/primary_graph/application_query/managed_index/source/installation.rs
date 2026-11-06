//! Readiness carries the actual configured catalog and compiled Native associations.
use super::super::{configuration::ManagedIndexProviderCatalog, source::service::SourceService};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::runtime::ManagedIndexSourceProgram;
use {
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial as Denial,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout,
};

#[derive(Default)]
pub struct ManagedSourceInstallation {
    sources: BTreeMap<ManagedIndexSourceProgram, Arc<dyn SourceService>>,
    public_cache_owner: bool,
    predicate_routing: bool,
}
impl ManagedSourceInstallation {
    pub(in crate::domain_computation::primary_graph) fn has_predicate_routing(&self) -> bool {
        self.predicate_routing
    }
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.sources.is_empty() && !self.public_cache_owner
    }
    pub(in crate::domain_computation::primary_graph) fn compile<S: ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        providers: &ManagedIndexProviderCatalog<S>,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<
        (
            Self,
            Option<super::initialization::SourceInitializationIssuer>,
        ),
        Denial,
    > {
        #[expect(
            clippy::mutable_key_type,
            reason = "Programs compare only their immutable owner-minted order; audience custody cannot affect ordering"
        )]
        let mut required = BTreeMap::new();
        for source in layout.required_managed_sources() {
            let provider = providers
                .source_factory(&source.definition)
                .ok_or_else(|| {
                    Denial::new(
                        Kind::MissingManagedIndexProvider,
                        &source.definition.parts().name,
                    )
                })?;
            required.insert(source.program.clone(), (provider, source.cold));
        }
        if required.is_empty() {
            return Ok((
                Self {
                    sources: BTreeMap::new(),
                    public_cache_owner: !providers.is_empty(),
                    predicate_routing: false,
                },
                None,
            ));
        }
        let (issuer, authority) =
            super::initialization::SourceInitializationIssuer::install(execution)
                .map_err(super::initialization::initialization_execution_denial)?;
        #[expect(
            clippy::mutable_key_type,
            reason = "Programs compare only their immutable owner-minted order; audience custody cannot affect ordering"
        )]
        let sources = required
            .into_iter()
            .map(|(program, (provider, cold))| {
                Ok((
                    program.clone(),
                    provider.bind(execution, program, cold, authority.clone())?,
                ))
            })
            .collect::<Result<
                BTreeMap<_, _>,
                worth_relational::facade::transactions::RelationalExecutionDenialCause,
            >>()
            .map_err(super::initialization::initialization_execution_denial)?;
        let predicate_routing = sources
            .values()
            .any(|service| service.predicate_axis().is_some());
        Ok((
            Self {
                public_cache_owner: !providers.is_empty(),
                sources,
                predicate_routing,
            },
            Some(issuer),
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn dispose_predicates(&self) {
        for service in self.sources.values() {
            if let Some(axis) = service.predicate_axis() {
                axis.dispose();
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn service(
        &self,
        program: &ManagedIndexSourceProgram,
    ) -> Option<&Arc<dyn SourceService>> {
        self.sources.get(program)
    }
}
