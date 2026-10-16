//! The installed A -> B -> C graph uses actual successor discovery and producers.
use super::super::program as chain_program;
use super::super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationInputPort, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(super) struct TailFeature;
pub(super) struct TailSourceInput;
pub(super) struct TailOutput;
impl ApplicationFeature<CheckpointSchema> for TailFeature {
    type Inputs = ApplicationFeatureInputList<TailSourceInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.retained-chain-tail.v1";
}
impl ApplicationInputPort<CheckpointSchema, TailFeature> for TailSourceInput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<CheckpointSchema, TailFeature> for TailOutput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "body";
}
pub(super) struct ChainToChain;
impl ApplicationConnectionIdentity for ChainToChain {
    const IDENTITY: &'static str = "worth.query.certification.retained-chain-edge.v1";
}
impl
    ApplicationOccurrenceConnectionBinding<
        CheckpointSchema,
        chain_program::ChainFeature,
        TailFeature,
    > for ChainToChain
{
}
impl WorthQueryApplicationDependentOutputConnection<CheckpointSchema> for ChainToChain {
    type RootDemand = ChainDemand;
    type Discovery = PlanarOutputRead;
    type Demand = ChainDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &ChainDemand,
    ) -> Result<PlanarOutputRead, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputRead {
            body_key: root.0.clone(),
        })
    }
    fn demands_from_discovery(
        discovery: &PlanarOutputReadResult,
    ) -> Result<Vec<ChainDemand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![ChainDemand(discovery.successor_body_key.clone())])
    }
}
pub(super) type NestedConnection = ApplicationConnectionRef<
    CheckpointSchema,
    chain_program::ChainFeature,
    chain_program::ChainOutput,
    TailFeature,
    TailSourceInput,
    ChainToChain,
>;
pub(super) type Root = ApplicationOutputGraph<
    RootConnection,
    ApplicationOutputEdge<
        chain_program::ChainConnection,
        ApplicationOutputEdge<NestedConnection, ApplicationOutputLeaf>,
    >,
>;
pub(super) struct Program;
impl ApplicationProgramDefinition<CheckpointSchema> for Program {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.retained-predecessors.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = chain_program::ChainProgram::feature_specs();
        features.push(
            ApplicationFeatureSpec::root::<CheckpointSchema, TailFeature>()
                .provides::<TailOutput>()
                .conditional_operation::<PublishChain>()
                .finish(),
        );
        features
    }
}
