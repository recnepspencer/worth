//! The actual authored source-to-replacement output connection.
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};

worth_query_decl::facade::worth_query_application! {
    pub(super) MixedSchema {
        owner: "worth.query.certification.mixed-retirement",
        version: (1, 0),
        contributions: [TopologyContribution],
    }
}
impl TopologySchemaBinding for MixedSchema {}
pub(super) struct ReplacementFeature;
pub(super) struct ReplacementInput;
pub(super) struct ReplacementOutput;
pub(super) struct SourceToReplacement;

impl ApplicationFeature<MixedSchema> for ReplacementFeature {
    type Inputs = ApplicationFeatureInputList<ReplacementInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.mixed-replacement-feature.v1";
}
impl ApplicationInputPort<MixedSchema, ReplacementFeature> for ReplacementInput {
    type Value = PlanarReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<MixedSchema, ReplacementFeature> for ReplacementOutput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "replacement";
}
impl ApplicationConnectionIdentity for SourceToReplacement {
    const IDENTITY: &'static str = "worth.query.certification.source-to-mixed-replacement.v1";
}
impl ApplicationOccurrenceConnectionBinding<MixedSchema, PlanarSourceFeature, ReplacementFeature>
    for SourceToReplacement
{
}
impl WorthQueryApplicationRequiredOutputConnection<MixedSchema> for SourceToReplacement {
    type Source = PlanarSourceAdjustmentBinding<MixedSchema>;
    type Demand = ReplacementDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(ReplacementDemand(source.scope_key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<MixedSchema, SourceToReplacement>
    for PlanarSourceAdjustmentBinding<MixedSchema>
{
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<ReplacementDemand, WorthQueryRequiredOutputConnectionDenial> {
        <SourceToReplacement as WorthQueryApplicationRequiredOutputConnection<MixedSchema>>::demand_from_source(source)
    }
}

type Connection = ApplicationConnectionRef<
    MixedSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    ReplacementFeature,
    ReplacementInput,
    SourceToReplacement,
>;
pub(super) type MixedRoot = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
type OrdinaryRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        MixedSchema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        PlanarOutputFeature,
        PlanarBodyInput,
        PlanarSourceToOutputConnection,
    >,
    (
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                MixedSchema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarFinalOutputFeature,
                PlanarDerivedBodyInput,
                PlanarOutputToFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                MixedSchema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarAlternateFinalOutputFeature,
                PlanarAlternateDerivedBodyInput,
                PlanarOutputToAlternateFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
    ),
>;
pub(super) struct MixedProgram;
impl ApplicationProgramDefinition<MixedSchema> for MixedProgram {
    type Contributions = <MixedSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<(MixedRoot, OrdinaryRoot)>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<MixedSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.mixed-retirement-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<MixedSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .mutation::<PlanarSourceAdjustmentBinding<MixedSchema>>()
                .mutation::<PlanarEditBinding<MixedSchema>>()
                .finish(),
            required_chain::output_feature_spec_for::<MixedSchema>(),
            ApplicationFeatureSpec::root::<MixedSchema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PreserveFinalPlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<MixedSchema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<MixedSchema, ReplacementFeature>()
                .provides::<ReplacementOutput>()
                .conditional_operation::<ReplacePlanarVertex>()
                .finish(),
        ]
    }
}
