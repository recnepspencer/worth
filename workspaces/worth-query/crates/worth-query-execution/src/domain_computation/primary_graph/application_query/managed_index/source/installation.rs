//! Readiness carries the actual configured catalog and compiled Native associations.
use super::super::{configuration::ManagedIndexProviderCatalog, source::service::SourceService};
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout,
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::runtime::ManagedIndexSourceProgram;

#[derive(Default)]
pub struct ManagedSourceInstallation {
    sources: HashMap<ManagedIndexSourceProgram, usize>,
    ordered_sources: Vec<Arc<dyn SourceService>>,
    _allocation:
        Option<Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>>,
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
                    sources: HashMap::new(),
                    ordered_sources: Vec::new(),
                    _allocation: None,
                    public_cache_owner: !providers.is_empty(),
                    predicate_routing: false,
                },
                None,
            ));
        }
        let (issuer, authority) =
            super::initialization::SourceInitializationIssuer::install(execution)
                .map_err(super::initialization::initialization_execution_denial)?;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        let allocation =
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(
                execution,
            )
            .map_err(super::initialization::initialization_execution_denial)?;
        allocation
            .claim_slice::<Arc<dyn SourceService>>(execution, required.len())
            .map_err(super::initialization::initialization_execution_denial)?;
        let mut ordered_sources = Vec::new();
        ordered_sources
            .try_reserve_exact(required.len())
            .map_err(|_| {
                super::initialization::initialization_execution_denial(Cause::AllocationFailed)
            })?;
        let mut sources = HashMap::new();
        let mut predicate_routing = false;
        // The layout owns declaration order. Hash storage supplies point lookup only.
        // Removing by program preserves the original deduplication and last provider.
        for source in layout.required_managed_sources() {
            let Some((provider, cold)) = required.remove(&source.program) else {
                continue;
            };
            let service = provider
                .bind(execution, source.program.clone(), cold, authority.clone())
                .map_err(super::initialization::initialization_execution_denial)?;
            predicate_routing |= service.predicate_axis().is_some();
            sources.insert(source.program.clone(), ordered_sources.len());
            ordered_sources.push(service);
        }
        Ok((
            Self {
                public_cache_owner: !providers.is_empty(),
                sources,
                ordered_sources,
                _allocation: Some(allocation),
                predicate_routing,
            },
            Some(issuer),
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn dispose_predicates(&self) {
        for service in &self.ordered_sources {
            if let Some(axis) = service.predicate_axis() {
                axis.dispose();
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn service(
        &self,
        program: &ManagedIndexSourceProgram,
    ) -> Option<&Arc<dyn SourceService>> {
        self.sources
            .get(program)
            .map(|index| &self.ordered_sources[*index])
    }
}
