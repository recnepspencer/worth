use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::super::handler::PendingMutationHandlerRegistry;
use super::super::{
    WorthQueryApplicationInvariantFactories, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::conditional::PendingConditionalRegistry;
use super::contracts::WorthQueryApplicationContractCatalog;
use super::producer::WorthQueryInstalledApplicationProducerRegistry;
use super::WorthQueryApplicationContributionContracts;
use super::WorthQueryApplicationContributionSetup;

/// Entry-owned configuration for the members declared by one contribution.
pub trait WorthQueryApplicationContribution<Schema>: ApplicationSchemaContribution<Schema>
where
    Schema: ApplicationSchema,
{
    type Configuration;

    fn contracts(
        _contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>;
}

mod tuple_seal {
    pub trait Sealed {}
}

/// Sealed traversal of the root's single contribution list and its matching configuration tuple.
pub trait WorthQueryApplicationContributionTuple<Schema>: tuple_seal::Sealed
where
    Schema: ApplicationSchema,
{
    type Configuration;

    #[doc(hidden)]
    fn configure(
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        configuration: Self::Configuration,
        contracts: WorthQueryApplicationContractCatalog<Schema>,
    ) -> Result<
        WorthQueryConfiguredApplicationContributions<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    >;

    #[doc(hidden)]
    fn contracts() -> Result<
        WorthQueryApplicationContractCatalog<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    >;
}

/// Move-only pending handlers and factories validated against an installed root.
pub struct WorthQueryConfiguredApplicationContributions<Schema>
where
    Schema: ApplicationSchema,
{
    factories: WorthQueryApplicationInvariantFactories<Schema>,
    handlers: PendingMutationHandlerRegistry<Schema>,
    producers: super::producer::PendingProducerRegistry<Schema>,
    conditionals: PendingConditionalRegistry<Schema>,
}

impl<Schema> WorthQueryConfiguredApplicationContributions<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn configure<Contributions>(
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
        contracts: WorthQueryApplicationContractCatalog<Schema>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial>
    where
        Contributions: WorthQueryApplicationContributionTuple<Schema>,
    {
        Contributions::configure(installed_schema, configuration, contracts)
    }
}

impl<Schema> WorthQueryConfiguredApplicationContributions<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> Result<
        (
            WorthQueryApplicationInvariantFactories<Schema>,
            PendingMutationHandlerRegistry<Schema>,
            WorthQueryInstalledApplicationProducerRegistry<Schema>,
            PendingConditionalRegistry<Schema>,
        ),
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        Ok((
            self.factories,
            self.handlers,
            self.producers.seal()?,
            self.conditionals,
        ))
    }
}

fn validate_inventory<Schema: ApplicationSchema>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    mut expected: Vec<ApplicationSchemaContributionIdentity>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    expected.sort();
    let installed = installed_schema
        .contributions()
        .iter()
        .map(|row| row.identity().clone())
        .collect::<Vec<_>>();
    if expected.windows(2).any(|pair| pair[0] == pair[1]) || expected != installed {
        return Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::ContributionInventoryMismatch,
            "root contribution tuple differs from the installed contribution catalog",
        ));
    }
    Ok(())
}

macro_rules! contribution_tuple {
    ($($Contribution:ident: $configuration:ident),+) => {
        impl<$($Contribution),+> tuple_seal::Sealed for ($($Contribution,)+) {}

        impl<Schema, $($Contribution),+> WorthQueryApplicationContributionTuple<Schema>
            for ($($Contribution,)+)
        where
            Schema: ApplicationSchema,
            $($Contribution: WorthQueryApplicationContribution<Schema>,)+
        {
            type Configuration = ($($Contribution::Configuration,)+);

            fn configure(
                installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
                configuration: Self::Configuration,
                contracts: WorthQueryApplicationContractCatalog<Schema>,
            ) -> Result<WorthQueryConfiguredApplicationContributions<Schema>, WorthQueryPrimaryGraphInstallationDenial> {
                validate_inventory(installed_schema, vec![$($Contribution::IDENTITY,)+])?;
                let (producers, conditionals) = contracts.into_pending();
                let mut configured = WorthQueryConfiguredApplicationContributions {
                    factories: WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema),
                    handlers: PendingMutationHandlerRegistry::default(),
                    producers,
                    conditionals,
                };
                let ($($configuration,)+) = configuration;
                $(
                    let contribution = installed_schema.contributions().get($Contribution::IDENTITY.as_str())
                        .expect("exact contribution inventory validated before callbacks");
                    let mut setup = WorthQueryApplicationContributionSetup::new(
                        installed_schema, contribution, &mut configured.handlers, &mut configured.factories,
                        &mut configured.producers,
                        &mut configured.conditionals,
                    );
                    $Contribution::configure($configuration, &mut setup)?;
                )+
                configured.handlers.seal(installed_schema, &installed_schema.binding_identity())?;
                configured.producers.validate_complete()?;
                configured.conditionals.validate_complete()?;
                Ok(configured)
            }

            fn contracts() -> Result<WorthQueryApplicationContractCatalog<Schema>, WorthQueryPrimaryGraphInstallationDenial> {
                let mut catalog = WorthQueryApplicationContractCatalog::default();
                $(
                    let mut contribution = WorthQueryApplicationContributionContracts::for_contribution(
                        $Contribution::IDENTITY.as_str(),
                    );
                    $Contribution::contracts(&mut contribution)?;
                    contribution.append_to(&mut catalog)?;
                )+
                catalog.validate()?;
                Ok(catalog)
            }
        }
    };
}

contribution_tuple!(A: a);
contribution_tuple!(A: a, B: b);
contribution_tuple!(A: a, B: b, C: c);
contribution_tuple!(A: a, B: b, C: c, D: d);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j, K: k);
contribution_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j, K: k, L: l);
