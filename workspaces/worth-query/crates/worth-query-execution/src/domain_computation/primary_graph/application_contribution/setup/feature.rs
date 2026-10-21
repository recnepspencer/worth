//! The existing contribution setup consumes the feature's typed configuration.
use super::super::feature::{
    self, WorthQueryFeatureConfiguration, WorthQueryRequiredFeatureContract,
};
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'_, Schema> {
    pub fn feature<Contract>(
        &mut self,
        configuration: impl WorthQueryFeatureConfiguration<Schema, Contract>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Contract: WorthQueryRequiredFeatureContract<Schema>,
    {
        feature::install::<Schema, Contract>(configuration, self)
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn feature_conditional<
        Producer,
        Binding,
    >(
        &mut self,
        configuration: Binding::Configuration,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
        Binding: WorthQueryApplicationConditionalBinding<
            Schema,
            Operation = <Producer::Operation as ApplicationMutationBinding<Schema>>::Operation,
        >,
    {
        self.conditionals.register_with_requirements::<Binding>(
            self.contribution.identity().as_str(),
            configuration,
            feature::required_producers::<Schema, Producer, Binding>(),
        )
    }
}
