//! One installed provider with optional inverse routing and row-local refinement.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    managed_index::{refinement::RefinementCallbacks, ManagedIndexProviderCatalog},
    WorthQueryManagedIndexPredicateProvider, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexRefinementProvider,
};
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;

type Installer<S, I, P> = fn(
    &mut ManagedIndexProviderCatalog<S>,
    &WorthQueryInstalledApplicationSchema<S>,
    &str,
    P,
    Option<RefinementCallbacks<S, I, P>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>;

/// Composes optional mechanics over one provider object and one installation authority.
pub struct WorthQueryManagedIndexProviderRegistration<'registration, 'setup, S, I, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    setup: &'registration mut WorthQueryApplicationContributionSetup<'setup, S>,
    provider: P,
    refinement: Option<RefinementCallbacks<S, I, P>>,
    install: Installer<S, I, P>,
}
impl<S, I, P> WorthQueryManagedIndexProviderRegistration<'_, '_, S, I, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub fn with_predicate_routing(mut self) -> Self
    where
        P: WorthQueryManagedIndexPredicateProvider<S, I>,
    {
        self.install = ManagedIndexProviderCatalog::register_predicate::<I, P>;
        self
    }
    pub fn with_refinement(mut self) -> Self
    where
        P: WorthQueryManagedIndexRefinementProvider<S, I>,
    {
        self.refinement = Some(RefinementCallbacks::new());
        self
    }
    pub fn install(self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        self.setup.require_owned_query::<I::SourceBinding>()?;
        (self.install)(
            self.setup.managed_indexes,
            self.setup.installed_schema,
            self.setup.contribution.identity().as_str(),
            self.provider,
            self.refinement,
        )
    }
}
impl<'setup, S: ApplicationSchema + 'static> WorthQueryApplicationContributionSetup<'setup, S> {
    pub fn managed_index_provider<I, P>(
        &mut self,
        provider: P,
    ) -> WorthQueryManagedIndexProviderRegistration<'_, 'setup, S, I, P>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>,
    {
        WorthQueryManagedIndexProviderRegistration {
            setup: self,
            provider,
            refinement: None,
            install: ManagedIndexProviderCatalog::register::<I, P>,
        }
    }
    /// Installs inverse routing on the same actual provider object.
    pub fn managed_index_with_predicate_routing<I, P>(
        &mut self,
        provider: P,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexPredicateProvider<S, I>,
    {
        self.managed_index_provider::<I, P>(provider)
            .with_predicate_routing()
            .install()
    }
    /// Installs the declared candidate provider without optional mechanics.
    pub fn managed_index<I, P>(
        &mut self,
        provider: P,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>,
    {
        self.managed_index_provider::<I, P>(provider).install()
    }
}
