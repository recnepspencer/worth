//! Readiness carries the actual configured catalog and compiled Native associations.
use super::super::{configuration::ManagedIndexProviderCatalog, source::service::SourceService};
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout,
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use std::{collections::HashMap, sync::Arc};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationInvariantManagedIndexSource, ApplicationManagedIndexColdSource as Cold,
    },
    application_schema::ApplicationSchema,
};
use worth_relational::facade::runtime::ManagedIndexSourceProgram;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ManagedSourceInstallation {
    sources: HashMap<ManagedIndexSourceProgram, Arc<dyn SourceService>>,
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
        requirements: impl Iterator<Item = ApplicationInvariantManagedIndexSource>,
    ) -> Result<
        (
            Self,
            Option<super::initialization::SourceInitializationIssuer>,
        ),
        Denial,
    > {
        let mut required = HashMap::new();
        for requirement in requirements {
            let program = layout.managed_source(requirement.index()).ok_or_else(|| {
                Denial::new(Kind::UnsupportedInvariantSource, requirement.index())
            })?;
            let provider = providers
                .source_factory(requirement.definition())
                .ok_or_else(|| {
                    Denial::new(Kind::MissingManagedIndexProvider, requirement.index())
                })?;
            let entry = required
                .entry(program.clone())
                .or_insert((provider, Cold::AdmitRebuild));
            // One shared preparation cannot broaden a refusing audience by
            // borrowing the admission selected by another installed rule.
            if requirement.cold() == Cold::Refuse {
                entry.1 = Cold::Refuse;
            }
        }
        if required.is_empty() {
            return Ok((
                Self {
                    sources: HashMap::new(),
                    public_cache_owner: !providers.is_empty(),
                    predicate_routing: false,
                },
                None,
            ));
        }
        let (issuer, authority) =
            super::initialization::SourceInitializationIssuer::install(execution)
                .map_err(super::initialization::initialization_execution_denial)?;
        let sources = required
            .into_iter()
            .map(|(program, (provider, cold))| {
                Ok((
                    program.clone(),
                    provider.bind(execution, program, cold, authority.clone())?,
                ))
            })
            .collect::<Result<
                HashMap<_, _>,
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
