//! The declaring contribution installs the exact provider over disclosed source rows.
use super::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexProvider;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;

impl<S: ApplicationSchema + 'static> WorthQueryApplicationContributionSetup<'_, S> {
    /// Installs inverse routing on the same actual provider object. Whole-Parent
    /// watches still require their installed tracked profile and a maintained Source.
    pub fn managed_index_with_predicate_routing<I, P>(
        &mut self, provider: P,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where I: ApplicationManagedIndexBinding<S>,
        P: crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexPredicateProvider<S, I>,
{
        self.require_owned_query::<I::SourceBinding>()?;
        self.managed_indexes.register_predicate::<I, P>(
            self.installed_schema,
            self.contribution.identity().as_str(),
            provider,
        )
    }

    /// Installs this contribution's exact provider. Sources and consuming
    /// Parents currently require Public authorization and Public disclosure
    /// without capability/rule governance; unsupported Parents refuse at binding.
    pub fn managed_index<I, P>(
        &mut self,
        provider: P,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>,
    {
        self.require_owned_query::<I::SourceBinding>()?;
        self.managed_indexes.register::<I, P>(
            self.installed_schema,
            self.contribution.identity().as_str(),
            provider,
        )
    }
}
