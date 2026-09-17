//! A dependency set discovered by an evaluation is installed with
//! `set_evaluated_dependencies`: the node keeps the value it just computed
//! instead of being demoted to `MaybeStale` and recomputed on its next read,
//! which is what `set_dependencies` (a topology replacement under a value the
//! graph cannot vouch for) must do.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::facade::*;
use crate::tests::support::*;

type Upstreams = Arc<Mutex<BTreeMap<NodeId, Vec<NodeId>>>>;
type CallLog = Arc<Mutex<BTreeMap<NodeId, u64>>>;

struct World {
    runtime: SignalRuntime<(), (), (), (), ()>,
    left: NodeId,
    right: NodeId,
    node: NodeId,
    upstreams: Upstreams,
    calls: CallLog,
}

/// `node` reads `left` and `right`; the evaluator re-reads whatever the
/// upstream map says, so the test can shrink the read set the way a dynamic
/// callback does.
fn build() -> World {
    let mut graph = SignalGraph::new();
    let left = graph.node().produces_aspects(mask_a()).build();
    let right = graph.node().produces_aspects(mask_a()).build();
    let node = graph.node().produces_aspects(mask_a()).build();
    graph.append_dependency(node, left, ASPECT_A).unwrap();
    graph.append_dependency(node, right, ASPECT_A).unwrap();
    World {
        runtime: SignalRuntime::builder(graph).with_kernel_defaults().build(),
        left,
        right,
        node,
        upstreams: Arc::new(Mutex::new(BTreeMap::from([(node, vec![left, right])]))),
        calls: Arc::default(),
    }
}

fn evaluator(
    calls: CallLog,
    upstreams: Upstreams,
) -> impl Fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError> + Sync {
    move |context| {
        let reads = upstreams
            .lock()
            .expect("upstream map mutex poisoned")
            .get(&context.node())
            .cloned()
            .unwrap_or_default();
        for upstream in reads {
            context.read(upstream, ASPECT_A)?;
        }
        let mut calls = calls.lock().expect("call log mutex poisoned");
        let count = calls.entry(context.node()).or_insert(0);
        *count += 1;
        Ok(context.finish(version_ab(*count, 0)))
    }
}

fn calls_for(calls: &CallLog, node: NodeId) -> u64 {
    calls
        .lock()
        .expect("call log mutex poisoned")
        .get(&node)
        .copied()
        .unwrap_or(0)
}

fn state(world: &World, node: NodeId) -> NodeState {
    world
        .runtime
        .observe()
        .graph()
        .graph()
        .get_state(node)
        .unwrap()
}

fn read(world: &mut World, node: NodeId) {
    let evaluator = evaluator(world.calls.clone(), world.upstreams.clone());
    world.runtime.read(node, &(), &evaluator).unwrap();
}

fn settle(world: &mut World) {
    read(world, world.left);
    read(world, world.right);
    read(world, world.node);
}

fn commit_change(world: &mut World, source: NodeId) {
    let evaluator = evaluator(world.calls.clone(), world.upstreams.clone());
    world
        .runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            Ok(())
        })
        .unwrap();
}

fn shrink_reads_to_left(world: &World) {
    world
        .upstreams
        .lock()
        .expect("upstream map mutex poisoned")
        .insert(world.node, vec![world.left]);
}

#[test]
fn evaluated_dependencies_keep_the_node_clean_and_drop_the_removed_edge() {
    let mut world = build();
    settle(&mut world);
    assert_eq!(state(&world, world.node), NodeState::Clean);
    assert_eq!(calls_for(&world.calls, world.node), 1);

    // The evaluation that just ran read only `left`.
    shrink_reads_to_left(&world);
    let (node, left, right) = (world.node, world.left, world.right);
    world
        .runtime
        .graph_mut()
        .set_evaluated_dependencies(node, [DependencyEdge::new(left, ASPECT_A)])
        .unwrap();

    assert_eq!(state(&world, node), NodeState::Clean);
    {
        let graph = world.runtime.observe().graph().graph();
        assert_eq!(
            graph.dependencies_of(node).unwrap(),
            &[DependencyEdge::new(left, ASPECT_A)]
        );
        let left_version = graph.node_aspect_version(left).unwrap().get(ASPECT_A);
        let snapshot = graph.get_dep_snapshot(node).unwrap();
        assert_eq!(snapshot.entries().len(), 1);
        assert_eq!(snapshot.entries()[0].source, left);
        assert_eq!(snapshot.entries()[0].cached_version, left_version);
    }

    // A read right after the patch reuses the value: nothing recomputes.
    read(&mut world, node);
    assert_eq!(calls_for(&world.calls, node), 1);

    // The dropped source no longer reaches the node: with the edge still
    // installed the commit would leave it `MaybeStale` for revalidation.
    commit_change(&mut world, right);
    assert_eq!(state(&world, node), NodeState::Clean);
    read(&mut world, node);
    assert_eq!(calls_for(&world.calls, node), 1);

    // The kept source still does.
    commit_change(&mut world, left);
    read(&mut world, node);
    assert_eq!(calls_for(&world.calls, node), 2);
}

#[test]
fn plain_set_dependencies_demotes_the_node_so_the_next_read_recomputes() {
    // The contrast that motivates the evaluated variant.
    let mut world = build();
    settle(&mut world);
    let (node, left) = (world.node, world.left);
    world
        .runtime
        .graph_mut()
        .set_dependencies(node, [DependencyEdge::new(left, ASPECT_A)])
        .unwrap();
    assert_eq!(state(&world, node), NodeState::MaybeStale);
    shrink_reads_to_left(&world);
    read(&mut world, node);
    assert_eq!(calls_for(&world.calls, node), 2);
}

#[test]
fn evaluated_dependencies_do_not_launder_an_invalidation_that_landed_after_the_evaluation() {
    let mut world = build();
    settle(&mut world);
    let (node, left) = (world.node, world.left);
    // An invalidation reaches the node itself after its evaluation ran.
    crate::facade::core::mark_dirty(world.runtime.graph_mut(), node, ASPECT_A).unwrap();
    assert_ne!(state(&world, node), NodeState::Clean);

    world
        .runtime
        .graph_mut()
        .set_evaluated_dependencies(node, [DependencyEdge::new(left, ASPECT_A)])
        .unwrap();
    assert_ne!(
        state(&world, node),
        NodeState::Clean,
        "a node dirtied after its evaluation stays due for recompute"
    );
    shrink_reads_to_left(&world);
    read(&mut world, node);
    assert_eq!(calls_for(&world.calls, node), 2);
}

#[test]
fn evaluated_dependencies_inside_a_transaction_survive_commit_clean() {
    let mut world = build();
    settle(&mut world);
    let (node, left, right) = (world.node, world.left, world.right);
    let evaluator = evaluator(world.calls.clone(), world.upstreams.clone());
    world
        .runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(right, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            // `node` is standing demand: the transaction recomputes it, and
            // that recompute read only `left`, so install that topology.
            tx.evaluate_demand(&evaluator, &[node])?;
            tx.set_evaluated_dependencies(node, [DependencyEdge::new(left, ASPECT_A)])
        })
        .unwrap();
    assert_eq!(calls_for(&world.calls, node), 2);
    assert_eq!(state(&world, node), NodeState::Clean);
    assert_eq!(
        world
            .runtime
            .observe()
            .graph()
            .graph()
            .dependencies_of(node)
            .unwrap(),
        &[DependencyEdge::new(left, ASPECT_A)]
    );
    shrink_reads_to_left(&world);
    read(&mut world, node);
    assert_eq!(
        calls_for(&world.calls, node),
        2,
        "commit kept the evaluated value"
    );
    commit_change(&mut world, right);
    read(&mut world, node);
    assert_eq!(
        calls_for(&world.calls, node),
        2,
        "the removed edge is gone after commit"
    );
}
