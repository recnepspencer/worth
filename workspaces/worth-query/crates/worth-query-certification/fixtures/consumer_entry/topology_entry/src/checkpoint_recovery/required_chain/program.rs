//! Public program selection for the actual producers in the consumed-output test.
//!
//! The connection admits independent downstream demands. The A/B/C dependency
//! facts still come from the handler's actual `current_output` reads.

use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationInputPort, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(super) struct ChainFeature;
pub(super) struct ChainSourceInput;
pub(super) struct ChainOutput;
pub(super) struct ConsumeOutputConnection;

impl ApplicationFeature<CheckpointSchema> for ChainFeature {
    type Inputs = ApplicationFeatureInputList<ChainSourceInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-feature.v1";
}

impl ApplicationInputPort<CheckpointSchema, ChainFeature> for ChainSourceInput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl ApplicationOutputPort<CheckpointSchema, ChainFeature> for ChainOutput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "body";
}

impl ApplicationConnectionIdentity for ConsumeOutputConnection {
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-connection.v1";
}

impl ApplicationOccurrenceConnectionBinding<CheckpointSchema, PlanarOutputFeature, ChainFeature>
    for ConsumeOutputConnection
{
}

impl WorthQueryApplicationDependentOutputConnection<CheckpointSchema> for ConsumeOutputConnection {
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarOutputRead;
    type Demand = ChainDemand;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-connection.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputRead {
            body_key: root.body_key().to_owned(),
        })
    }

    fn demands_from_discovery(
        discovery: &PlanarOutputReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        // Geometric successors belong to the Source query. Required output
        // occurrences follow the declared consumed-output graph instead.
        Ok(producer::root_downstream(&discovery.body_key)
            .map(ChainDemand)
            .into_iter()
            .collect())
    }
}

pub(super) type ChainConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    ChainFeature,
    ChainSourceInput,
    ConsumeOutputConnection,
>;

pub(super) type ChainRoot = ApplicationOutputGraph<
    RootConnection,
    (
        (
            ApplicationOutputEdge<FinalConnection, ApplicationOutputLeaf>,
            ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
        ),
        ApplicationOutputEdge<ChainConnection, ApplicationOutputLeaf>,
    ),
>;

pub(super) struct ChainProgram;

impl ApplicationProgramDefinition<CheckpointSchema> for ChainProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ChainRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.consumed-chain-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = demand_policy::feature_specs_with_output::<demand_policy::FinalArtifact>(
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .native_action::<PlanarEditBinding<CheckpointSchema>>()
                .finish_action()
                .native_action::<VertexReplacementBinding<CheckpointSchema>>()
                .finish_action()
                .conditional_operation::<MutatePlanar>()
                .finish(),
        );
        features.push(
            ApplicationFeatureSpec::root::<CheckpointSchema, ChainFeature>()
                .provides::<ChainOutput>()
                .conditional_operation::<PublishChain>()
                .finish(),
        );
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_selects_consumers_instead_of_geometric_successors() {
        for (root, geometric_successor, expected) in [
            ("anchor-a", "anchor-source-b", Some("anchor-b")),
            ("diamond-left", "diamond-left-b", Some("diamond-join")),
            ("diamond-right", "diamond-right-b", Some("diamond-join")),
            ("ring-2-a", "ring-2-source-b", Some("ring-2-b")),
            ("anchor-source-b", "anchor-source-c", None),
            ("diamond-join", "diamond-join-b", None),
            ("ring-2-b", "ring-2-c", None),
            ("ring-2-c", "ring-2-a", None),
        ] {
            let observed = PlanarOutputReadResult {
                body_key: root.into(),
                successor_body_key: geometric_successor.into(),
                value: length(2),
            };
            let demands = ConsumeOutputConnection::demands_from_discovery(&observed).unwrap();
            assert_eq!(
                demands
                    .iter()
                    .map(|demand| demand.0.as_str())
                    .collect::<Vec<_>>(),
                expected.into_iter().collect::<Vec<_>>(),
                "required output occurrences for {root}"
            );
        }
    }
}
