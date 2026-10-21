use super::*;

worth_query_decl::facade::worth_query_application! {
    pub ExampleSchema {
        owner: "worth.query.example.required-feature", version: (1, 0),
        contributions: [TopologyContribution],
    }
}
impl TopologySchemaBinding for ExampleSchema {}

type SourceConnection = ApplicationConnectionRef<
    ExampleSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
pub type FinalConnection = ApplicationConnectionRef<
    ExampleSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
type AlternateConnection = ApplicationConnectionRef<
    ExampleSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;
pub type Root = ApplicationOutputGraph<
    SourceConnection,
    (
        ApplicationOutputEdge<FinalConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
    ),
>;
pub struct ExampleProgram<const ERASE: bool>;
impl<const ERASE: bool> ApplicationProgramDefinition<ExampleSchema> for ExampleProgram<ERASE> {
    type Contributions = (ExampleContribution<ERASE>,);
    type Outputs = ApplicationProgramOutputs<Root>;
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
            ApplicationFeatureSpec::root::<ExampleSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarSourceAdjustmentBinding<ExampleSchema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<ExampleSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<MutatePlanar>()
                .finish(),
            ApplicationFeatureSpec::root::<ExampleSchema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .required_pair::<PlanarFinalFeatureRoles<ExampleSchema>>()
                .finish(),
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
    type Configuration = TopologyConfiguration;
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
            configuration.setup_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        } else {
            TopologyContribution::configure(configuration, setup)
        }
    }
}
