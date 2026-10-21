//! Two real sibling layouts over the existing A -> B -> C producer contract.
use super::super::super::program as chain;
use super::super::super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationInputPort, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(super) struct SiblingFeature;
pub(super) struct SiblingInput;
pub(super) struct SiblingOutput;
impl ApplicationFeature<CheckpointSchema> for SiblingFeature {
    type Inputs = ApplicationFeatureInputList<SiblingInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.sibling-chain-feature.v1";
}
impl ApplicationInputPort<CheckpointSchema, SiblingFeature> for SiblingInput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<CheckpointSchema, SiblingFeature> for SiblingOutput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "body";
}

pub(super) struct ReverseChildren;
pub(super) struct EarlierConsumer;
pub(super) struct LaterProducer;
macro_rules! connection {
    ($binding:ident, $feature:ty, $identity:literal, [$($key:literal),+]) => {
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<CheckpointSchema, PlanarOutputFeature, $feature> for $binding {}
        impl WorthQueryApplicationDependentOutputConnection<CheckpointSchema> for $binding {
            type RootDemand = PlanarOutputDemand;
            type Discovery = PlanarOutputRead;
            type Demand = ChainDemand;
            const IDENTITY: &'static str = $identity;
            fn discovery_from_root(root: &PlanarOutputDemand) -> Result<PlanarOutputRead, WorthQueryRequiredOutputConnectionDenial> {
                Ok(PlanarOutputRead { body_key: root.body_key().to_owned() })
            }
            fn demands_from_discovery(_: &PlanarOutputReadResult) -> Result<Vec<ChainDemand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(vec![$(ChainDemand($key.into())),+])
            }
        }
    };
}
connection!(
    ReverseChildren,
    chain::ChainFeature,
    "sibling-reverse-edge",
    ["anchor-c", "anchor-b"]
);
connection!(
    EarlierConsumer,
    SiblingFeature,
    "sibling-earlier-consumer",
    ["anchor-c"]
);
connection!(
    LaterProducer,
    chain::ChainFeature,
    "sibling-later-producer",
    ["anchor-b"]
);

pub(super) type ReverseConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    chain::ChainFeature,
    chain::ChainSourceInput,
    ReverseChildren,
>;
pub(super) type EarlierConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    SiblingFeature,
    SiblingInput,
    EarlierConsumer,
>;
pub(super) type LaterConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    chain::ChainFeature,
    chain::ChainSourceInput,
    LaterProducer,
>;
pub(super) type EdgeRoot = ApplicationOutputGraph<
    RootConnection,
    ApplicationOutputEdge<ReverseConnection, ApplicationOutputLeaf>,
>;
pub(super) type BranchRoot = ApplicationOutputGraph<
    RootConnection,
    (
        ApplicationOutputEdge<EarlierConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<LaterConnection, ApplicationOutputLeaf>,
    ),
>;

pub(super) struct EdgeProgram;
pub(super) struct BranchProgram;
macro_rules! program {
    ($program:ident, $root:ty, $identity:literal, $tail:expr) => {
        impl ApplicationProgramDefinition<CheckpointSchema> for $program {
            type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
            type Outputs = ApplicationProgramOutputs<$root>;
            type Rules = CheckpointRules;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                // The selected program needs one planar root and its chain
                // children. Keep the contribution's other conditional operations
                // installed without declaring unused input-bearing features.
                let mut features = vec![
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
                ];
                if $tail {
                    features.push(
                        ApplicationFeatureSpec::root::<CheckpointSchema, SiblingFeature>()
                            .provides::<SiblingOutput>()
                            .conditional_operation::<PublishChain>()
                            .finish(),
                    );
                }
                features
            }
        }
    };
}
program!(
    EdgeProgram,
    EdgeRoot,
    "sibling-edge-progress-program",
    false
);
program!(
    BranchProgram,
    BranchRoot,
    "sibling-branch-progress-program",
    true
);
