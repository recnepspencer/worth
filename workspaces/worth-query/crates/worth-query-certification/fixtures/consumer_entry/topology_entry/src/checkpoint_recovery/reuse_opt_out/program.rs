//! Required final artifact with separate Initial and Preserve bindings.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
    ApplicationFeature,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;

type RootConnection = ApplicationConnectionRef<
    NoReuseSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
pub(super) type FinalConnection = ApplicationConnectionRef<
    NoReuseSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
type AlternateConnection = ApplicationConnectionRef<
    NoReuseSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;
pub(super) type Root = ApplicationOutputGraph<
    RootConnection,
    (
        ApplicationOutputEdge<FinalConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
    ),
>;
type Rules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<NoReuseSchema, PositivePlanarTurn>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

struct RequiredFinal;
impl ApplicationDerivedArtifact<NoReuseSchema, PlanarFinalOutputFeature> for RequiredFinal {
    type Output = PlanarFinalBodyOutput;
    type Locality = super::super::demand_policy::PlanarLocality;
    const IDENTITY: &'static str = "no-reuse-final-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession =
        ApplicationArtifactSuccession::PreserveWhenEquivalent;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <PlanarFinalOutputFamily as WorthQueryProducerOutputFamily<NoReuseSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarOutputFeature as ApplicationFeature<NoReuseSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4_096, 8_192);
    const STOPPED_OUTCOME: &'static str = "final-output-resource-denied";
}

pub(super) struct NoReuseProgram;
impl ApplicationProgramDefinition<NoReuseSchema> for NoReuseProgram {
    type Contributions = <NoReuseSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.no-reuse-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<NoReuseSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .mutation::<PlanarEditBinding<NoReuseSchema>>()
                .finish(),
            super::super::required_chain::output_feature_spec_for::<NoReuseSchema>(),
            ApplicationFeatureSpec::root::<NoReuseSchema, PlanarFinalOutputFeature>()
                .derived_artifact::<RequiredFinal>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PreserveFinalPlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<NoReuseSchema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
        ]
    }
}
