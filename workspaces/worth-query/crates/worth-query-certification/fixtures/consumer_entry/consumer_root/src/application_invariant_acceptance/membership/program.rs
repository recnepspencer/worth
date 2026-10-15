//! Real program connections and operation authorization for the separate membership schema.
use super::MembershipSchema as Schema;
use worth_query_decl::facade::{
    application_program::*, application_schema::ApplicationSchemaComposition,
};
use worth_query_parameter_entry::{ParameterFeature, PositiveParameterCount};
use worth_query_topology_entry::*;

pub(crate) struct Fanout;
pub(crate) struct Deep;
impl ApplicationCompositionInstance for Fanout {
    const PATH: &'static str = "membership.fanout";
}
impl ApplicationCompositionInstance for Deep {
    const PATH: &'static str = "membership.deep";
}

type FixedConnection = ApplicationConnectionRef<
    Schema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
type FinalConnection = ApplicationConnectionRef<
    Schema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
type SummaryConnection = ApplicationConnectionRef<
    Schema,
    PlanarFinalOutputFeature,
    PlanarFinalBodyOutput,
    PlanarSummaryFeature,
    PlanarSummaryInput,
    PlanarFinalToSummaryConnection,
>;
type AlternateConnection = ApplicationConnectionRef<
    Schema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;
type AlternateSummaryConnection = ApplicationConnectionRef<
    Schema,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateFinalBodyOutput,
    PlanarAlternateSummaryFeature,
    PlanarAlternateSummaryInput,
    PlanarAlternateFinalToSummaryConnection,
>;
type FixedRoot = ApplicationOutputGraph<
    FixedConnection,
    (
        ApplicationOutputEdge<
            FinalConnection,
            ApplicationOutputEdge<SummaryConnection, ApplicationOutputLeaf>,
        >,
        ApplicationOutputEdge<
            AlternateConnection,
            ApplicationOutputEdge<AlternateSummaryConnection, ApplicationOutputLeaf>,
        >,
    ),
>;

type FanoutConnection = ApplicationConnectionInstanceRef<
    Schema,
    Fanout,
    PlanarSourceFeature,
    PlanarBodyOutput,
    Fanout,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
type FanoutFinal = ApplicationConnectionInstanceRef<
    Schema,
    Fanout,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    Fanout,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToLateFinalConnection,
>;
pub(crate) type FanoutRoot = ApplicationDiscoveredOutputGraph<
    FanoutConnection,
    ApplicationOutputEdge<FanoutFinal, ApplicationOutputLeaf>,
>;

type DeepConnection = ApplicationConnectionInstanceRef<
    Schema,
    Deep,
    PlanarSourceFeature,
    PlanarBodyOutput,
    Deep,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
type MembershipConnection = ApplicationConnectionInstanceRef<
    Schema,
    Deep,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    Deep,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToMembershipConnection,
>;
type DeepSummary = ApplicationConnectionInstanceRef<
    Schema,
    Deep,
    PlanarFinalOutputFeature,
    PlanarFinalBodyOutput,
    Deep,
    PlanarSummaryFeature,
    PlanarSummaryInput,
    PlanarFinalToSummaryConnection,
>;
pub(crate) type MembershipRoot = ApplicationDiscoveredOutputGraph<
    DeepConnection,
    ApplicationOutputEdge<
        MembershipConnection,
        ApplicationOutputEdge<DeepSummary, ApplicationOutputLeaf>,
    >,
>;

type Rules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<Schema, PositivePlanarTurn>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, ParameterFeature, PositiveParameterCount>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >,
>;
pub(crate) struct MembershipProgram;
impl ApplicationProgramDefinition<Schema> for MembershipProgram {
    type Contributions = <Schema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<(FixedRoot, (FanoutRoot, MembershipRoot))>;
    type Rules = Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.membership-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarSourceAdjustmentBinding<Schema>>()
                .finish_action()
                .native_action::<PlanarEditBinding<Schema>>()
                .finish_action()
                .native_action::<PlanarDiscoveryLinkBinding<Schema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<MutatePlanar>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PreserveFinalPlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarSummaryFeature>().finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarAlternateSummaryFeature>().finish(),
            ApplicationFeatureSpec::root::<Schema, ParameterFeature>().finish(),
            ApplicationFeatureSpec::at::<Schema, Fanout, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Fanout, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Fanout, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Deep, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Deep, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Deep, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<Schema, Deep, PlanarSummaryFeature>().finish(),
        ]
    }
}
