//! The installed A -> B -> C graph uses actual successor discovery and producers.
use super::super::program as chain_program;
use super::super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationOccurrenceConnectionBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(super) struct ChainToChain;
impl ApplicationConnectionIdentity for ChainToChain {
    const IDENTITY: &'static str = "worth.query.certification.retained-chain-edge.v1";
}
impl
    ApplicationOccurrenceConnectionBinding<
        CheckpointSchema,
        chain_program::ChainFeature,
        chain_program::ChainFeature,
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
    chain_program::ChainFeature,
    chain_program::ChainSourceInput,
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
        chain_program::ChainProgram::feature_specs()
    }
}
