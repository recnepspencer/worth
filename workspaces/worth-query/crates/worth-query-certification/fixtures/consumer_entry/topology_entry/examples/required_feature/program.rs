use super::*;

worth_query_decl::facade::worth_query_application! {
    pub ExampleSchema {
        owner: "worth.query.example.required-feature", version: (1, 0),
        contributions: [TopologyContribution],
    }
}
impl TopologySchemaBinding for ExampleSchema {}

pub type FinalConnection = ApplicationConnectionRef<
    ExampleSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
pub type Root = PlanarSourceFeatureRoot<ExampleSchema>;
pub struct ExampleConfiguration {
    pub topology: TopologyConfiguration,
    pub source: PlanarSourceRuntime<ExampleSchema>,
}
pub struct ExampleProgram<const ERASE: bool>;
impl<const ERASE: bool> ApplicationProgramDefinition<ExampleSchema> for ExampleProgram<ERASE> {
    type Contributions = (ExampleContribution<ERASE>,);
    type Outputs = ApplicationProgramOutputs<
        <PlanarSourceFeatureDefinition<ExampleSchema> as ApplicationFeatureDefinition<
            ExampleSchema,
        >>::Outputs,
    >;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<ExampleSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("required-feature-example");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            <PlanarSourceFeatureDefinition<ExampleSchema> as ApplicationFeatureDefinition<
                ExampleSchema,
            >>::feature_specs()
            .pop()
            .expect("one complete source feature"),
            ApplicationFeatureSpec::root::<ExampleSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<MutatePlanar>()
                .finish(),
            <PlanarFinalFeatureDefinition<ExampleSchema> as ApplicationFeatureDefinition<
                ExampleSchema,
            >>::feature_specs()
            .pop()
            .expect("one complete final feature"),
            ApplicationFeatureSpec::root::<ExampleSchema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
        ]
    }
}

// The negative keeps exactly the same schema and program requirements. Only
// the contribution's contract/configuration callbacks omit the feature.
pub struct ExampleContribution<const ERASE: bool>;
impl<const ERASE: bool> ApplicationSchemaContribution<ExampleSchema>
    for ExampleContribution<ERASE>
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<ExampleSchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<ExampleSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<ExampleSchema> {
        <TopologyContribution as ApplicationSchemaContribution<ExampleSchema>>::register_members(
            builder,
        )
    }
}
impl<const ERASE: bool> WorthQueryApplicationContribution<ExampleSchema>
    for ExampleContribution<ERASE>
{
    type Configuration = ExampleConfiguration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<ExampleSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        if ERASE {
            Ok(())
        } else {
            TopologyContribution::contracts(contracts)
        }
    }
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, ExampleSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        if ERASE {
            configuration
                .topology
                .setup_calls
                .fetch_add(1, Ordering::SeqCst);
            Ok(())
        } else {
            TopologyContribution::configure_with_source(
                configuration.topology,
                configuration.source,
                setup,
            )
        }
    }
}
