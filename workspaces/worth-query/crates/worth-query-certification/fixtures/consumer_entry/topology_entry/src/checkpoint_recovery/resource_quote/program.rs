//! The counted producer uses ordinary program/source/output declarations.
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};
pub struct SourceFeature;
pub struct OutputFeature;
pub struct Output;
pub struct InputPort;
pub struct Link;
pub struct Locality;
pub struct Artifact;
pub struct Program;
impl<S: TopologySchemaBinding> ApplicationFeature<S> for SourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "quote-count.source-feature";
}
impl<S: TopologySchemaBinding> ApplicationFeature<S> for OutputFeature {
    type Inputs = ApplicationFeatureInputList<InputPort, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "quote-count.output-feature";
}
impl<S: TopologySchemaBinding> ApplicationOutputPort<S, SourceFeature> for Output {
    type Value = ResultValue;
    const IDENTITY: &'static str = "count-source";
}
impl<S: TopologySchemaBinding> ApplicationOutputPort<S, OutputFeature> for Output {
    type Value = ResultValue;
    const IDENTITY: &'static str = "count-output";
}
impl<S: TopologySchemaBinding> ApplicationInputPort<S, OutputFeature> for InputPort {
    type Value = ResultValue;
    const IDENTITY: &'static str = "count-input";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for Link {
    const IDENTITY: &'static str = "quote-count.connection";
}
impl<S: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<S, SourceFeature, OutputFeature> for Link
{
}
impl<S: TopologySchemaBinding> WorthQueryApplicationRequiredOutputConnection<S> for Link {
    type Source = EditBinding<S>;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = "quote-count.connection";
    fn demand_from_source(
        source: &Edit,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand(source.scope_key.clone()))
    }
}
impl<S: TopologySchemaBinding> WorthQueryApplicationRequiredOutputSource<S, Link>
    for EditBinding<S>
{
    fn demand_from_source(
        source: &Edit,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand(source.scope_key.clone()))
    }
}
pub type Root = ApplicationDemandOutputGraph<
    ApplicationConnectionRef<QuoteSchema, SourceFeature, Output, OutputFeature, InputPort, Link>,
    ApplicationOutputLeaf,
>;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "quote-count.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
struct SourceChange;
impl ApplicationChangeShape for SourceChange {
    const IDENTITY: &'static str = "quote-count.source-reconstruction";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Reconstruct;
}
impl ApplicationDerivedArtifact<QuoteSchema, OutputFeature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "quote-count.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "quote-count.family";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <SourceFeature as ApplicationFeature<QuoteSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";

    const STOPPED_OUTCOME: &'static str = "counted-output-stopped";
}
impl ApplicationProgramDefinition<QuoteSchema> for Program {
    type Contributions = <QuoteSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("quote-count.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<QuoteSchema, SourceFeature>()
                .provides::<Output>()
                .native_action::<EditBinding<QuoteSchema>>()
                .locality_and_change::<Locality, SourceChange>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<QuoteSchema, OutputFeature>()
                .derived_artifact::<Artifact>()
                .conditional_operation::<Produce>()
                .finish(),
        ]
    }
}
