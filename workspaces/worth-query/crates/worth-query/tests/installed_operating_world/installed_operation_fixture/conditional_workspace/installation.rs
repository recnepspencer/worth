use worth_query::facade::{consumer_kit, domain};
use worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner;
use worth_runtime_bridge::facade::{RelationalBridgeRecordIdentityParts, RuntimeBridge};

use super::super::correspondence_bridge::conditional_runtime_bridge_from_source;
use super::providers::providers_for;

pub(crate) struct ConditionalInstallation {
    semantic: domain::WorthQuerySemanticTruthDependency,
    observation: ConditionalObservationFixture,
    pub(crate) graph: worth_signal::facade::SignalGraph,
    pub(crate) node_identity: String,
    pub(crate) signal_node: worth_signal::facade::NodeId,
    pub(crate) dependency: domain::WorthQueryConditionalDependencyInstallation,
    pub(crate) providers: worth_runtime_bridge::facade::BridgeConditionalProviderSet,
}

#[derive(Clone, Copy)]
enum ConditionalObservationFixture {
    SeededRecord,
    Missing,
}

pub(crate) struct PreparedConditionalInstallation {
    pub(crate) owner: WorthQueryRelationalSourceOwner,
    pub(crate) bridge: RuntimeBridge,
    pub(crate) graph: worth_signal::facade::SignalGraph,
    pub(crate) node_identity: String,
    pub(crate) record: RelationalBridgeRecordIdentityParts,
    pub(crate) dependency: domain::WorthQueryConditionalDependencyInstallation,
    pub(crate) providers: worth_runtime_bridge::facade::BridgeConditionalProviderSet,
}

impl ConditionalInstallation {
    pub(crate) fn semantic(&self) -> &domain::WorthQuerySemanticTruthDependency {
        &self.semantic
    }

    pub(crate) fn prepare_seeded(
        self,
        owner: &WorthQueryRelationalSourceOwner,
        seed: &consumer_kit::WorthQueryTestSeedReceipt,
    ) -> PreparedConditionalInstallation {
        let record = seed
            .relational_record(super::source_seed::SOURCE_KEY)
            .expect("conditional source row was committed before installation");
        let bridge = conditional_runtime_bridge_from_source(owner, &self.semantic, record);
        self.prepare(owner.clone(), bridge, record)
    }

    pub(crate) fn prepare_public(self) -> PreparedConditionalInstallation {
        let (owner, seed) = super::source_seed::standalone(self.semantic.clone());
        self.prepare_seeded(&owner, &seed)
    }

    fn prepare(
        self,
        owner: WorthQueryRelationalSourceOwner,
        bridge: RuntimeBridge,
        record: RelationalBridgeRecordIdentityParts,
    ) -> PreparedConditionalInstallation {
        let source_record = match self.semantic.locality() {
            domain::WorthQuerySemanticLocality::SourceRecord => Some(record),
            _ => None,
        };
        let dependency = domain::WorthQueryConditionalDependencyInstallation::new(
            source_record,
            self.dependency.signal_targets().to_vec(),
        );
        let dependency = match self.observation {
            ConditionalObservationFixture::SeededRecord => {
                dependency.with_observation_record(record)
            }
            ConditionalObservationFixture::Missing => dependency,
        };
        PreparedConditionalInstallation {
            owner,
            bridge,
            graph: self.graph,
            node_identity: self.node_identity,
            record,
            dependency,
            providers: self.providers,
        }
    }
}

pub(crate) fn conditional_installation(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
) -> ConditionalInstallation {
    declaration(
        node,
        "geometry-signal",
        ConditionalObservationFixture::SeededRecord,
    )
}

pub(crate) fn conditional_installation_without_observation(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
) -> ConditionalInstallation {
    declaration(
        node,
        "geometry-signal",
        ConditionalObservationFixture::Missing,
    )
}

pub(super) fn conditional_installation_pair_in_partitions(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
    current_partition: &str,
    candidate_partition: &str,
) -> (ConditionalInstallation, ConditionalInstallation) {
    (
        declaration(
            node,
            current_partition,
            ConditionalObservationFixture::SeededRecord,
        ),
        declaration(
            node,
            candidate_partition,
            ConditionalObservationFixture::SeededRecord,
        ),
    )
}

fn declaration(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
    partition: &str,
    observation: ConditionalObservationFixture,
) -> ConditionalInstallation {
    let mut graph = worth_signal::facade::SignalGraph::new();
    let signal_node_id = graph.node().build();
    let worth_proof::TransitionOutcome::Success(signal_node) =
        graph.admit_installed_node(signal_node_id)
    else {
        panic!("fresh Signal node should admit")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new(partition),
        signal_node,
    );
    ConditionalInstallation {
        semantic: node.dependencies()[0].clone(),
        observation,
        graph,
        node_identity: node.identity().to_string(),
        signal_node: signal_node_id,
        dependency: domain::WorthQueryConditionalDependencyInstallation::new(None, vec![target]),
        providers: providers_for(node),
    }
}
