//! One later-producer/earlier-consumer sibling layout over A -> B -> C.
use super::super::super::program as chain;
use super::super::super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationOccurrenceConnectionBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(super) struct ReverseChildren;
impl ApplicationConnectionIdentity for ReverseChildren {
    const IDENTITY: &'static str = "sibling-reverse-edge";
}
impl
    ApplicationOccurrenceConnectionBinding<
        CheckpointSchema,
        PlanarOutputFeature,
        chain::ChainFeature,
    > for ReverseChildren
{
}
impl WorthQueryApplicationDependentOutputConnection<CheckpointSchema> for ReverseChildren {
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarOutputRead;
    type Demand = ChainDemand;
    const IDENTITY: &'static str = "sibling-reverse-edge";
    fn discovery_from_root(
        root: &PlanarOutputDemand,
    ) -> Result<PlanarOutputRead, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputRead {
            body_key: root.body_key().to_owned(),
        })
    }
    fn demands_from_discovery(
        _: &PlanarOutputReadResult,
    ) -> Result<Vec<ChainDemand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![
            ChainDemand("anchor-c".into()),
            ChainDemand("anchor-b".into()),
        ])
    }
}
pub(super) type ReverseConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    chain::ChainFeature,
    chain::ChainSourceInput,
    ReverseChildren,
>;
pub(super) type EdgeRoot = ApplicationOutputGraph<
    RootConnection,
    ApplicationOutputEdge<ReverseConnection, ApplicationOutputLeaf>,
>;
pub(super) struct EdgeProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for EdgeProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<EdgeRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("sibling-edge-progress-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarEditBinding<CheckpointSchema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .native_action::<PlanarEditBinding<CheckpointSchema>>()
                .finish_action()
                .conditional_operation::<MutatePlanar>()
                .required_pair::<PlanarFinalFeatureRoles<CheckpointSchema, PlanarOutputFeature>>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<CheckpointSchema, chain::ChainFeature>()
                .provides::<chain::ChainOutput>()
                .conditional_operation::<PublishChain>()
                .finish(),
        ]
    }
}
