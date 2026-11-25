//! Single-declaration macro collapsing `ApplicationFeature`,
//! `ApplicationSchemaContribution`, `Registration`,
//! `WorthQueryApplicationContribution`, and `ApplicationProgramDefinition`.

/// Declares a single-contribution application program from one block,
/// lowering to the exact public feature, contribution and program traits.
///
/// Use the inline `feature: Feature => Definition / Runtime { identity: .., .. }`
/// form to declare the feature and its runtime beside the contribution and
/// program, or `feature_definition: ExistingDefinition,` to wrap a feature
/// already declared with [`worth_query_application_contribution!`].
///
/// A contribution may also install conditional bindings, such as desktop
/// temporal rules, with
/// `conditionals: Configuration { field: Binding, .. },` after `members`.
/// Each binding joins the contribution's members (it registers its own
/// operation), contributes its contract, gains a
/// [`WorthQueryConditionalFeature`](crate::facade::application_contribution::WorthQueryConditionalFeature)
/// root, and is configured from the generated `Configuration { feature, field, .. }`
/// struct, whose `feature` field holds the feature runtime.
#[macro_export]
macro_rules! worth_query_application_program {
    (
        $vis:vis program $Program:ident for $Schema:ty {
            identity: $program_identity:expr,
            $(rules: $Rules:ty,)?
            contribution: $Contribution:ident {
                identity: $contribution_identity:expr,
                members: [$($Member:ty),* $(,)?],
                $(conditionals: $Config:ident { $($field:ident: $Conditional:ty),* $(,)? },)?
            },
            $(managed_indexes: [$($ManagedIndex:ty),* $(,)?],)?
            feature: $Feature:ident => $Definition:ident / $Runtime:ident {
                identity: $feature_identity:expr,
                $($feature_block:tt)*
            }
        }
    ) => {
        $vis struct $Feature;
        impl $crate::facade::declaration::application_program::ApplicationFeature<$Schema>
            for $Feature
        {
            type Inputs =
                $crate::facade::declaration::application_program::ApplicationFeatureInputLeaf;
            const IDENTITY: &'static str = $feature_identity;
        }
        $crate::worth_query_application_contribution! {
            $vis feature $Definition<$Schema> for $Feature => $Runtime {
                $($feature_block)*
                $(managed_indexes: [$($ManagedIndex,)*];)?
            }
        }
        $crate::worth_query_application_program! {
            $vis program $Program for $Schema {
                identity: $program_identity,
                $(rules: $Rules,)?
                contribution: $Contribution {
                    identity: $contribution_identity,
                    members: [$($Member,)*],
                    $(conditionals: $Config { $($field: $Conditional,)* },)?
                },
                feature_definition: $Definition,
            }
        }
    };
    (
        $vis:vis program $Program:ident for $Schema:ty {
            identity: $program_identity:expr,
            $(rules: $Rules:ty,)?
            contribution: $Contribution:ident {
                identity: $contribution_identity:expr,
                members: [$($Member:ty),* $(,)?],
                $(conditionals: $Config:ident { $($field:ident: $Conditional:ty),* $(,)? },)?
            },
            feature_definition: $Definition:ty $(,)?
        }
    ) => {
        $crate::worth_query_application_program!(@contribution
            [$vis $Contribution, $Schema, $Definition, $contribution_identity]
            [$($Member,)*] [$($Config [$($field: $Conditional,)*])?]
        );
        $vis struct $Program;
        impl $crate::facade::declaration::application_program::ApplicationProgramDefinition<$Schema>
            for $Program
        {
            type Contributions = ($Contribution,);
            type Outputs =
                $crate::facade::declaration::application_program::ApplicationProgramOutputs<
                    <$Definition as $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>>::Outputs,
                >;
            type Rules = $crate::worth_query_application_program!(@rules $($Rules)?);
            const IDENTITY: $crate::facade::declaration::application_program::ApplicationProgramIdentity =
                $crate::facade::declaration::application_program::ApplicationProgramIdentity::new(
                    $program_identity,
                );

            fn feature_specs() -> ::std::vec::Vec<
                $crate::facade::declaration::application_program::ApplicationFeatureSpec,
            > {
                #[allow(unused_mut)]
                let mut specs = <$Definition as $crate::facade::declaration::application_program::ApplicationFeatureDefinition<$Schema>>::feature_specs();
                $($(specs.push(
                    $crate::facade::application_contribution::WorthQueryConditionalFeature::<$Conditional>::feature_spec::<$Schema>(),
                );)*)?
                specs
            }
        }
    };
    (@contribution [$vis:vis $Contribution:ident, $Schema:ty, $Definition:ty, $contribution_identity:expr]
        [$($Member:ty,)*] []
    ) => {
        $vis struct $Contribution;
        impl $crate::facade::declaration::application_schema::ApplicationSchemaContribution<$Schema>
            for $Contribution
        {
            const IDENTITY: $crate::facade::declaration::application_schema::ApplicationSchemaContributionIdentity =
                $crate::facade::declaration::application_schema::ApplicationSchemaContributionIdentity::new(
                    $contribution_identity,
                );
            type Members = ($($Member,)* $Definition,);
        }
        impl $crate::facade::application_contribution::WorthQueryApplicationContribution<$Schema>
            for $Contribution
        {
            type Configuration =
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::Configuration;

            fn contracts(
                contracts: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionContracts<$Schema>,
            ) -> ::core::result::Result<
                (),
                $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
            > {
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::contracts(contracts)
            }

            fn configure(
                configuration: Self::Configuration,
                setup: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, $Schema>,
            ) -> ::core::result::Result<
                (),
                $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
            > {
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::configure(configuration, setup)
            }
        }
    };
    (@contribution [$vis:vis $Contribution:ident, $Schema:ty, $Definition:ty, $contribution_identity:expr]
        [$($Member:ty,)*] [$Config:ident [$($field:ident: $Conditional:ty,)*]]
    ) => {
        /// Contribution configuration: the feature runtime and each conditional
        /// binding's configuration.
        $vis struct $Config {
            pub feature:
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::Configuration,
            $(pub $field:
                <$Conditional as $crate::facade::application_contribution::WorthQueryApplicationConditionalBinding<$Schema>>::Configuration,)*
        }
        $vis struct $Contribution;
        impl $crate::facade::declaration::application_schema::ApplicationSchemaContribution<$Schema>
            for $Contribution
        {
            const IDENTITY: $crate::facade::declaration::application_schema::ApplicationSchemaContributionIdentity =
                $crate::facade::declaration::application_schema::ApplicationSchemaContributionIdentity::new(
                    $contribution_identity,
                );
            type Members = ($($Member,)* $($Conditional,)* $Definition,);
        }
        impl $crate::facade::application_contribution::WorthQueryApplicationContribution<$Schema>
            for $Contribution
        {
            type Configuration = $Config;

            fn contracts(
                contracts: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionContracts<$Schema>,
            ) -> ::core::result::Result<
                (),
                $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
            > {
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::contracts(contracts)?;
                $(contracts.conditional::<$Conditional>()?;)*
                ::core::result::Result::Ok(())
            }

            fn configure(
                configuration: Self::Configuration,
                setup: &mut $crate::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, $Schema>,
            ) -> ::core::result::Result<
                (),
                $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
            > {
                let $Config { feature, $($field,)* } = configuration;
                <$Definition as $crate::facade::application_contribution::WorthQueryFeatureDefinition<$Schema>>::configure(feature, setup)?;
                $(setup.conditional::<$Conditional>($field)?;)*
                ::core::result::Result::Ok(())
            }
        }
    };
    (@rules $Rules:ty) => { $Rules };
    (@rules) => { $crate::facade::declaration::application_program::ApplicationRuleLeaf };
}
