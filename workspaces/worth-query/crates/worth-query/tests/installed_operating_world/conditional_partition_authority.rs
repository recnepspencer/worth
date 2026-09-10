use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::conditional_node_contract::node;
use super::installed_operation_fixture::{
    conditional_workspace, correspondence_bridge, ConditionalModelGraph, GeometryDomain,
    ReadFamily, ReadVertex,
};

#[test]
fn partition_dependency_crosses_the_real_relational_source_and_signal_delivery_path() {
    let partition = worth_foundational::facade::TruthPartitionRole::new("model-main").unwrap();
    let workspace = conditional_workspace(
        "partition-correspondence",
        node(
            "partition-node",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourcePartition(partition),
        ),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let operating_world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let graph_participation = workspace
        .graph_participation(ConditionalModelGraph)
        .unwrap();
    let location = domain::WorthQueryConditionalNodeLocation::operation("partition-node").unwrap();
    let mut signal_graph = worth_signal::facade::SignalGraph::new();
    let signal_node = signal_graph.node().build();
    let TransitionOutcome::Success(signal_node_capability) =
        signal_graph.admit_installed_node(signal_node)
    else {
        panic!("installed Signal node capability")
    };
    let aspect = worth_signal::facade::Aspect::new(0);
    let TransitionOutcome::Success(aspect_capability) =
        signal_graph.admit_installed_aspect(signal_node, aspect)
    else {
        panic!("installed Signal aspect capability")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::exact(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new("geometry-signal"),
        signal_node_capability,
        aspect_capability,
    )
    .unwrap();
    let registration = operation
        .semantic_correspondence_registration(
            location.clone(),
            0,
            &graph_participation,
            None,
            vec![target],
        )
        .unwrap();
    let (bridge, request) = correspondence_bridge(registration);
    {
        let mut graph_binding = bridge.bind_signal_graph(&mut signal_graph).unwrap();

        let TransitionOutcome::Success(correspondence) = operation.install_semantic_correspondence(
            location,
            0,
            &graph_participation,
            None,
            &mut graph_binding,
        ) else {
            panic!("partition correspondence installs")
        };
        assert_eq!(
            correspondence
                .admission_counters()
                .partition_widened_matches(),
            1
        );
        let TransitionOutcome::Success(counters) =
            correspondence.deliver_authoritative_change(&mut graph_binding, request)
        else {
            panic!("partition-scoped Relational publication delivers")
        };
        assert_eq!(counters.truth_targets_admitted(), 1);
        assert_eq!(counters.signal_seeds_emitted(), 1);
    }
    assert_eq!(
        signal_graph
            .node_aspect_version(signal_node)
            .unwrap()
            .get(aspect),
        1
    );
}
