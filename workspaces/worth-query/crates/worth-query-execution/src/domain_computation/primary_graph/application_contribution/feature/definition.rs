//! A static feature definition uses the existing contribution installation owners.
use super::super::{
    WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;
use worth_query_declaration::facade::{
    application_program::ApplicationFeatureDefinition, application_schema::ApplicationSchema,
};

pub trait WorthQueryFeatureDefinition<S: ApplicationSchema>:
    ApplicationFeatureDefinition<S>
{
    type Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>;
}
impl<S, A, B> WorthQueryFeatureDefinition<S> for (A, B)
where
    S: ApplicationSchema,
    A: WorthQueryFeatureDefinition<S>,
    B: WorthQueryFeatureDefinition<S>,
{
    type Configuration = (A::Configuration, B::Configuration);
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        A::contracts(contracts)?;
        B::contracts(contracts)
    }
    fn configure(
        (a, b): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        A::configure(a, setup)?;
        B::configure(b, setup)
    }
}
