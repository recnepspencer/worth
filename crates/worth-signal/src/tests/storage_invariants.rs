use crate::facade::*;
use crate::tests::support::*;
use stats_alloc::{Region, INSTRUMENTED_SYSTEM};
use std::env;
use std::process::Command;

const ROLLBACK_COST_TEST: &str =
    "tests::storage_invariants::rollback_one_created_node_does_not_scan_shared_free_list";

#[test]
fn rollback_one_created_node_does_not_scan_shared_free_list() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_ROLLBACK_COST_CHILD";
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(ROLLBACK_COST_TEST)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated rollback allocation probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(
            output.status.success(),
            "isolated rollback allocation probe failed"
        );
        assert!(
            stdout.contains(ROLLBACK_COST_TEST)
                && stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "isolated rollback probe must execute exactly one named test"
        );
        return;
    }

    let mut samples = Vec::new();
    for free_slot_count in [64_usize, 4_096, 65_536] {
        let mut source = SignalGraph::new();
        let nodes = (0..=free_slot_count)
            .map(|_| source.create_node())
            .collect::<Vec<_>>();
        let anchor = nodes[free_slot_count];
        source.rollback_created_nodes(&nodes[..free_slot_count]);
        let expected_free_list = source.free_list_snapshot();
        let expected_active_nodes = source.active_node_count();
        assert_eq!(expected_free_list.len(), free_slot_count);

        let (mut fork, _) = source.fork_persistent();
        let created = fork.create_node();
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        fork.rollback_created_nodes(&[created]);
        let allocation = region.change();
        samples.push((
            free_slot_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert!(source.is_alive(anchor), "source anchor remains live");
        assert!(fork.is_alive(anchor), "fork anchor remains live");
        assert!(!fork.is_alive(created), "rolled-back node must be dead");
        assert_eq!(source.active_node_count(), expected_active_nodes);
        assert_eq!(fork.active_node_count(), expected_active_nodes);
        assert_eq!(source.free_list_snapshot(), expected_free_list);
        assert_eq!(fork.free_list_snapshot(), expected_free_list);
    }

    let minimum_calls = samples.iter().map(|(_, calls, _)| *calls).min().unwrap();
    let minimum_bytes = samples.iter().map(|(_, _, bytes)| *bytes).min().unwrap();
    for (free_slot_count, calls, bytes) in samples {
        assert!(
            calls <= minimum_calls + 32,
            "one-node rollback calls slope with {free_slot_count} free slots: {calls} vs {minimum_calls}"
        );
        assert!(
            bytes <= minimum_bytes + 32 * 1_024,
            "one-node rollback bytes slope with {free_slot_count} free slots: {bytes} vs {minimum_bytes}"
        );
    }
}

#[test]
fn rollback_created_nodes_keeps_free_list_unique_and_bounded_across_reuse_cycles() {
    let mut graph = SignalGraph::new();
    let anchor = graph.create_node();
    let reclaimed = graph.create_node();
    graph.unregister_node(reclaimed).unwrap();

    for _ in 0..8 {
        let created = (0..6).map(|_| graph.create_node()).collect::<Vec<_>>();
        graph.rollback_created_nodes(&created);

        let free_list = graph.free_list_snapshot();
        let mut unique = free_list.clone();
        unique.sort_unstable();
        unique.dedup();

        assert_eq!(free_list.len(), unique.len());
        assert!(free_list
            .iter()
            .all(|index| (*index as usize) < graph.arena_capacity()));
        assert!(graph.is_alive(anchor));
        assert_eq!(graph.active_node_count(), 1);
    }
}

#[test]
fn rollback_tail_trim_preserves_preexisting_free_entries_and_checkpoint_truth() {
    let mut graph = SignalGraph::new();
    let nodes = (0..=4_096).map(|_| graph.create_node()).collect::<Vec<_>>();
    let anchor = nodes[0];
    for node in &nodes[1..] {
        graph.unregister_node(*node).unwrap();
    }

    let reused_tail = graph.create_node();
    assert_eq!(reused_tail.index(), 4_096);
    graph.rollback_created_nodes(&[reused_tail]);

    assert_eq!(graph.arena_capacity(), 4_096);
    assert!(graph.is_alive(anchor));
    assert!(graph
        .free_list_snapshot()
        .iter()
        .all(|index| (*index as usize) < graph.arena_capacity()));

    let authority = graph.capture_checkpoint_authority();
    let mut restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    assert!(restored
        .free_list_snapshot()
        .iter()
        .all(|index| (*index as usize) < restored.arena_capacity()));
    let replacement = restored.create_node();
    assert_eq!(replacement.index(), 4_095);
    assert!(restored.is_alive(replacement));
}

#[test]
fn changed_persistent_fork_serializes_and_restores_without_mutating_live_source() {
    let mut source = SignalGraph::new();
    let upstream = source.create_node();
    let downstream = source.create_node();
    source
        .set_dependencies(downstream, [DependencyEdge::new(upstream, Aspect::new(0))])
        .unwrap();
    source.set_node_state(downstream, NodeState::Clean).unwrap();

    let (mut fork, _) = source.fork_persistent();
    let fork_only = fork.create_node();
    fork.set_node_state(downstream, NodeState::Dirty).unwrap();
    fork.set_dependencies(fork_only, [DependencyEdge::new(downstream, Aspect::new(1))])
        .unwrap();

    let serialized = serde_json::to_vec(&fork).expect("forked graph serializes");
    let restored: SignalGraph =
        serde_json::from_slice(&serialized).expect("forked graph reconstitutes");
    assert_eq!(restored.get_state(downstream).unwrap(), NodeState::Dirty);
    assert!(restored.is_alive(fork_only));
    assert_eq!(
        restored.dependencies_of(fork_only).unwrap(),
        [DependencyEdge::new(downstream, Aspect::new(1))]
    );

    let authority = fork.capture_checkpoint_authority();
    let encoded_authority = serde_json::to_vec(&authority).expect("authority serializes");
    let decoded_authority: crate::state::SignalCheckpointAuthority =
        serde_json::from_slice(&encoded_authority).expect("authority reconstitutes");
    let checkpoint_restored =
        SignalGraph::restore_from_checkpoint_authority(&decoded_authority).unwrap();
    assert_eq!(
        checkpoint_restored.get_state(downstream).unwrap(),
        NodeState::Dirty
    );
    assert!(checkpoint_restored.is_alive(fork_only));
    assert_eq!(
        checkpoint_restored.dependencies_of(fork_only).unwrap(),
        [DependencyEdge::new(downstream, Aspect::new(1))]
    );

    assert_eq!(source.get_state(downstream).unwrap(), NodeState::Clean);
    assert!(!source.is_alive(fork_only));
    assert_eq!(
        source.dependencies_of(downstream).unwrap(),
        [DependencyEdge::new(upstream, Aspect::new(0))]
    );
}

#[test]
fn slot_reuse_after_unregister_does_not_inherit_stale_subscribers() {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let dependent = graph.create_node();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, source, &mut compute).unwrap();
    evaluate(&mut graph, dependent, &mut compute).unwrap();

    graph.unregister_node(source).unwrap();
    evaluate(&mut graph, dependent, &mut compute).unwrap();
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);

    let replacement = graph.create_node();
    assert_eq!(replacement.index(), source.index());
    assert!(graph.subscribers_of(replacement).unwrap().is_empty());

    mark_dirty(&mut graph, replacement, ASPECT_A).unwrap();
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);
}

#[test]
fn rebuild_subscriber_index_after_slot_reuse_matches_live_dependencies_only() {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let dependent = graph.create_node();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    graph.unregister_node(source).unwrap();
    let replacement = graph.create_node();
    graph.rebuild_subscriber_index_from_dependencies().unwrap();

    assert_eq!(replacement.index(), source.index());
    assert!(graph.subscribers_of(replacement).unwrap().is_empty());
    assert!(graph.runtime_dependencies_of(dependent).unwrap().is_empty());
}
