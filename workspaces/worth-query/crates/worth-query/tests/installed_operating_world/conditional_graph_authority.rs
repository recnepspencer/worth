use worth_query::facade::domain;

mod declaration_contracts;

use super::conditional_node_contract::{dependency, node};
use super::installed_operation_fixture::{
    conditional_workspace, correspondence_bridge, fixture_record_identity, ConditionalModelGraph,
    GeometryDomain, ReadFamily, ReadVertex,
};

fn geometry_node_location() -> domain::WorthQueryConditionalNodeLocation {
    domain::WorthQueryConditionalNodeLocation::operation("geometry").unwrap()
}

#[test]
fn installed_operation_and_exact_graph_participation_mint_the_candidate() {
    let declared = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let workspace = conditional_workspace(
        "installed-correspondence-candidate",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
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
    let graph = workspace
        .graph_participation(ConditionalModelGraph)
        .unwrap();
    let mut signal_graph = worth_signal::facade::SignalGraph::new();
    let signal_node = signal_graph.node().build();
    let worth_proof::TransitionOutcome::Success(signal_node) =
        signal_graph.admit_installed_node(signal_node)
    else {
        panic!("installed Signal node capability")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new("geometry-signal"),
        signal_node,
    );
    assert!(matches!(
        operation.semantic_correspondence_registration(
            geometry_node_location(),
            0,
            &graph,
            None,
            vec![target.clone()],
        ),
        Err(denial) if denial.kind()
            == worth_runtime_bridge::facade::BridgeCorrespondenceDenialKind::InvalidPortableDependency
    ));
    let registration = operation
        .semantic_correspondence_registration(
            geometry_node_location(),
            0,
            &graph,
            Some(fixture_record_identity()),
            vec![target.clone()],
        )
        .unwrap();
    let candidate = registration.dependency();

    assert_eq!(candidate.contract(), declared.contract());
    assert_eq!(candidate.binding(), declared.binding());
    assert_eq!(candidate.declared_graph_role(), "model");
    assert!(!candidate.graph_participation_identity().is_empty());
    assert!(!candidate.graph_adapter_identity().is_empty());

    assert!(matches!(
        operation.semantic_correspondence_registration(
            geometry_node_location(),
            1,
            &graph,
            Some(fixture_record_identity()),
            vec![target],
        ),
        Err(denial) if denial.kind()
            == worth_runtime_bridge::facade::BridgeCorrespondenceDenialKind::PortableDependencyNotOwnedByOperation
    ));
}

#[test]
fn bound_query_facade_installs_correspondence_with_operation_authority() {
    let workspace = conditional_workspace(
        "installed-correspondence-owner",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
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
    let mut signal_graph = worth_signal::facade::SignalGraph::new();
    let node = signal_graph.node().build();
    let worth_proof::TransitionOutcome::Success(node_capability) =
        signal_graph.admit_installed_node(node)
    else {
        panic!("installed Signal node capability")
    };
    let aspect = worth_signal::facade::Aspect::new(0);
    let worth_proof::TransitionOutcome::Success(aspect_capability) =
        signal_graph.admit_installed_aspect(node, aspect)
    else {
        panic!("installed Signal aspect capability")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::exact(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new("geometry-signal"),
        node_capability,
        aspect_capability,
    )
    .unwrap();
    let registration = operation
        .semantic_correspondence_registration(
            geometry_node_location(),
            0,
            &graph_participation,
            Some(fixture_record_identity()),
            vec![target],
        )
        .unwrap();
    let (bridge, publication_request) = correspondence_bridge(registration);
    {
        let mut graph_binding = bridge.bind_signal_graph(&mut signal_graph).unwrap();

        let worth_proof::TransitionOutcome::Success(installed) = operation
            .install_semantic_correspondence(
                geometry_node_location(),
                0,
                &graph_participation,
                Some(fixture_record_identity()),
                &mut graph_binding,
            )
        else {
            panic!("Query should retain the installed correspondence authority")
        };
        assert_eq!(installed.installation_generation(), 1);
        assert_eq!(installed.target_count(), 1);
        assert!(!installed.graph_participation_identity().is_empty());
        let worth_proof::TransitionOutcome::Success(counters) =
            installed.deliver_authoritative_change(&mut graph_binding, publication_request)
        else {
            panic!("the real Relational publication should drive Signal invalidation")
        };
        assert_eq!(counters.truth_targets_admitted(), 1);
        assert_eq!(counters.signal_seeds_emitted(), 1);
    }
    assert_eq!(
        signal_graph.node_aspect_version(node).unwrap().get(aspect),
        1
    );
}

#[test]
fn foreign_runtime_graph_participation_denies_before_bridge_admission() {
    let first = conditional_workspace(
        "candidate-runtime-first",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let second = conditional_workspace(
        "candidate-runtime-second",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let domain = first.domain(GeometryDomain).unwrap();
    let operating_world = first
        .observe_operating_world(first.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    let foreign_graph = second.graph_participation(ConditionalModelGraph).unwrap();

    assert!(matches!(
        operation.semantic_correspondence_registration(
            geometry_node_location(),
            0,
            &foreign_graph,
            Some(fixture_record_identity()),
            Vec::new(),
        ),
        Err(denial) if denial.kind()
            == worth_runtime_bridge::facade::BridgeCorrespondenceDenialKind::GraphParticipationNotOwnedByOperation
            && denial.counters()
                == worth_runtime_bridge::facade::CorrespondenceAdmissionCounters::zero()
    ));
}
