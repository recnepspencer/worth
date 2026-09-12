use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaComposition, ApplicationSchemaContribution,
    ApplicationSchemaContributionIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::super::handler::PendingMutationHandlerRegistry;
use super::super::{
    WorthQueryApplicationInvariantFactories, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::WorthQueryApplicationContributionSetup;

/// Entry-owned configuration for the members declared by one contribution.
pub trait WorthQueryApplicationContribution<Schema>: ApplicationSchemaContribution<Schema>
where
    Schema: ApplicationSchema,
{
    type Configuration;

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
    ) -> Result<
        WorthQueryConfiguredApplicationContributions<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    >;
}

/// Move-only pending handlers and factories validated against an installed root.
pub struct WorthQueryConfiguredApplicationContributions<Schema> {
    factories: WorthQueryApplicationInvariantFactories<Schema>,
    handlers: PendingMutationHandlerRegistry<Schema>,
}

impl<Schema> WorthQueryConfiguredApplicationContributions<Schema>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    pub fn configure(
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        Schema::Contributions::configure(installed_schema, configuration)
    }
}

impl<Schema> WorthQueryConfiguredApplicationContributions<Schema> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationInvariantFactories<Schema>,
        PendingMutationHandlerRegistry<Schema>,
    ) {
        (self.factories, self.handlers)
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
            ) -> Result<WorthQueryConfiguredApplicationContributions<Schema>, WorthQueryPrimaryGraphInstallationDenial> {
                validate_inventory(installed_schema, vec![$($Contribution::IDENTITY,)+])?;
                let mut configured = WorthQueryConfiguredApplicationContributions {
                    factories: WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema),
                    handlers: PendingMutationHandlerRegistry::default(),
                };
                let ($($configuration,)+) = configuration;
                $(
                    let contribution = installed_schema.contributions().get($Contribution::IDENTITY.as_str())
                        .expect("exact contribution inventory validated before callbacks");
                    let mut setup = WorthQueryApplicationContributionSetup::new(
                        installed_schema, contribution, &mut configured.handlers, &mut configured.factories,
                    );
                    $Contribution::configure($configuration, &mut setup)?;
                )+
                configured.handlers.seal(installed_schema, &installed_schema.binding_identity())?;
                Ok(configured)
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
