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
}
impl ManagedSourceInstallation {
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
    pub(in crate::domain_computation::primary_graph) fn compile<S: ApplicationSchema>(
        providers: &ManagedIndexProviderCatalog<S>,
        layout: &WorthQueryPrimaryGraphLayout,
        requirements: impl Iterator<Item = ApplicationInvariantManagedIndexSource>,
    ) -> Result<Self, Denial> {
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
        Ok(Self {
            sources: required
                .into_iter()
                .map(|(program, (provider, cold))| (program.clone(), provider.bind(program, cold)))
                .collect(),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn service(
        &self,
        program: &ManagedIndexSourceProgram,
    ) -> Option<&Arc<dyn SourceService>> {
        self.sources.get(program)
    }
}
