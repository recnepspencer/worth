use crate::data::aspect::Aspect;
use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

type TestRuntime = SignalRuntime<(), (), (), (), ()>;

fn populated_runtime() -> (TestRuntime, [crate::data::handle::NodeId; 4]) {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let berth = graph.create_node();
    let depot = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("the populated root dependency installs");
    (
        SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph),
        [weather, berth, depot, dispatch],
    )
}

#[test]
fn cross_branch_snapshot_restore_preserves_outgoing_and_displaces_target_state() {
    let (mut runtime, [weather, berth, depot, dispatch]) = populated_runtime();
    let root_snapshot = runtime
        .capture_snapshot()
        .expect("the root snapshot captures");
    let root = runtime.current_branch();
    let feature = runtime
        .create_branch("restore-selection-feature")
        .expect("the feature branch forks");
    let sibling = runtime
        .create_branch("restore-selection-sibling")
        .expect("the sibling branch forks");

    runtime
        .switch_branch(feature.clone())
        .expect("the feature activates");
    runtime
        .graph_mut()
        .set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
        .expect("the feature dependency changes");
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("the populated feature captures");
    assert_eq!(
        feature_snapshot
            .diagnostic_graph
            .dependency_sources_of(dispatch),
        Ok(vec![berth]),
        "the feature snapshot retains its older captured content"
    );
    runtime
        .graph_mut()
        .set_dependencies(dispatch, [DependencyEdge::new(depot, Aspect::new(0))])
        .expect("the active feature advances after its last capture");

    runtime
        .restore_snapshot(&root_snapshot)
        .expect("restoring the root snapshot selects its branch");
    assert_eq!(runtime.current_branch().id, root.id);
    assert_eq!(
        runtime.graph().dependency_sources_of(dispatch),
        Ok(vec![weather]),
        "the selected root restores its captured content"
    );
    runtime
        .switch_branch(feature.clone())
        .expect("the nonminimum feature is the actual selection at sealing");

    let (_, mutation, _) = runtime
        .owner_port_slots()
        .expect("one active and one stored state per live branch seal honestly");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    assert_eq!(
        owner.selected_branch_id(),
        feature.id,
        "sealing preserves the actual selected nonminimum branch identity"
    );
    let admission = owner.admit().expect("the owner admits state inspection");
    for (branch_id, expected_source) in [
        (feature.id, depot),
        (root.id, weather),
        (sibling.id, weather),
    ] {
        owner
            .lookup_cell(&admission, branch_id)
            .expect("every live branch owns one installed cell")
            .with_state(&admission, |state, _| {
                assert_eq!(
                    state.state().graph().dependency_sources_of(dispatch),
                    Ok(vec![expected_source]),
                    "direct sealing retains exactly the latest live content"
                );
            })
            .expect("the owner cell remains inspectable");
    }
}

#[test]
fn portable_snapshot_restore_uses_live_target_state_and_target_local_plan() {
    let (mut source, [_, source_berth, _, source_dispatch]) = populated_runtime();
    let source_feature = source
        .create_branch("portable-restore-feature")
        .expect("the source feature forks");
    source
        .switch_branch(source_feature)
        .expect("the source feature activates");
    source
        .graph_mut()
        .set_dependencies(
            source_dispatch,
            [DependencyEdge::new(source_berth, Aspect::new(0))],
        )
        .expect("the portable source dependency changes");
    let mut source_dependency_snapshot = DependencySnapshot::empty();
    source_dependency_snapshot.record(source_berth, Aspect::new(0), 9, None);
    source
        .graph_mut()
        .set_dep_snapshot(source_dispatch, source_dependency_snapshot)
        .expect("the portable snapshot records its dependency observation");
    let portable_snapshot = source
        .capture_snapshot()
        .expect("the source feature captures a portable snapshot");

    let (mut runtime, [weather, berth, _, dispatch]) = populated_runtime();
    let root = runtime.current_branch();
    let feature = runtime
        .create_branch("portable-restore-feature")
        .expect("the destination feature forks with the same identity");
    let sibling = runtime
        .create_branch("portable-restore-sibling")
        .expect("the destination sibling forks");
    assert_eq!(feature.id, portable_snapshot.meta.branch_id);
    runtime
        .switch_branch(feature.clone())
        .expect("the destination feature activates for target-local setup");
    let mut target_dependency_snapshot = DependencySnapshot::empty();
    target_dependency_snapshot.record(weather, Aspect::new(0), 7, None);
    runtime
        .graph_mut()
        .set_dep_snapshot(dispatch, target_dependency_snapshot)
        .expect("the live target records a distinct dependency observation");
    runtime
        .switch_branch(sibling.clone())
        .expect("the destination sibling activates");
    runtime
        .graph_mut()
        .set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
        .expect("the outgoing dependency already matches the portable snapshot");
    let mut outgoing_dependency_snapshot = DependencySnapshot::empty();
    outgoing_dependency_snapshot.record(berth, Aspect::new(0), 9, None);
    runtime
        .graph_mut()
        .set_dep_snapshot(dispatch, outgoing_dependency_snapshot)
        .expect("the outgoing dependency observation matches the portable snapshot");
    let sibling_only = runtime.graph_mut().create_node();

    runtime
        .restore_snapshot(&portable_snapshot)
        .expect("the portable snapshot selects its existing live target");
    assert_eq!(runtime.current_branch().id, feature.id);
    assert_eq!(
        runtime.graph().dependency_sources_of(dispatch),
        Ok(vec![berth])
    );
    let checkpoint = runtime.observe().metrics().checkpoint;
    assert_eq!(
        checkpoint.snapshot_restore_shared_delta_node_count, 1,
        "the live target's weather-to-berth delta, not the matching outgoing graph, owns planning"
    );
    assert_eq!(
        checkpoint.snapshot_restore_coarse_reason_count, 2,
        "the live target has no node-set difference from the portable snapshot"
    );

    let (_, mutation, _) = runtime
        .owner_port_slots()
        .expect("portable selection seals one state for every live branch");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("the owner admits state inspection");
    for (branch_id, expected_source, sibling_node_is_live) in [
        (feature.id, berth, false),
        (sibling.id, berth, true),
        (root.id, weather, false),
    ] {
        owner
            .lookup_cell(&admission, branch_id)
            .expect("every portable-selection branch owns one cell")
            .with_state(&admission, |state, _| {
                assert_eq!(
                    state.state().graph().dependency_sources_of(dispatch),
                    Ok(vec![expected_source])
                );
                assert_eq!(
                    state.state().graph().is_alive(sibling_only),
                    sibling_node_is_live,
                    "only the outgoing sibling retains its post-fork node"
                );
            })
            .expect("the portable-selection cell remains inspectable");
    }
}
