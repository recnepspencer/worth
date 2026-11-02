//! A second maintained provider uses genuine edge-to-vertex projection routes.
pub(super) mod authoring;
#[cfg(test)]
mod court;
pub(super) mod generated;
pub(super) mod provider;
pub(super) mod query;
pub(super) mod refinement;
pub(super) mod refresh;
pub(super) mod rule;
pub(super) mod schema;
use super::schema::Catalog;
use std::sync::Arc;
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexDefinition, application_schema::*,
};
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationContributionSetup,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub(super) struct Configuration {
    pub provider: Arc<provider::Calls>,
    pub rules: Arc<rule::Calls>,
}
impl Configuration {
    pub fn new(provider: Arc<provider::Calls>) -> Self {
        Self {
            rules: Arc::new(rule::Calls::new(Arc::clone(&provider))),
            provider,
        }
    }
}
impl Default for Configuration {
    fn default() -> Self {
        Self::new(Arc::new(provider::Calls::default()))
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    refresh::declare(authoring::declare(
        schema::declare(schema)
            .application_query(query::source_definition())
            .application_query_binding::<query::SourceBinding>()
            .application_query(refinement::definition())
            .application_query_binding::<refinement::Binding>()
            .application_query(query::box_definition())
            .application_query_binding::<query::BoxBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<
                Catalog,
                provider::BoxIndex,
            >::new())
            .invariant(rule::definition()),
    ))
}
pub(super) fn configure(
    configuration: Configuration,
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup
        .managed_index_provider::<provider::BoxIndex, _>(provider::Provider(Arc::clone(
            &configuration.provider,
        )))
        .with_predicate_routing()
        .with_refinement()
        .install()?;
    setup.handler::<authoring::Binding, _>(authoring::Handler(configuration.provider))?;
    setup.handler::<refresh::Binding, _>(refresh::Handler)?;
    setup.invariant(
        rule::ProbeContact::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        move |resolver| rule::resolve(resolver, Arc::clone(&configuration.rules)),
    )
}
