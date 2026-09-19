//! `refresh_retained_diagnostics_views` makes `history_now()` describe the
//! graph as it stands, the way a snapshot restore does, for a graph that was
//! rebuilt outside of a transaction. Under the operational tier
//! `history_now()` is the most recent retained history entry, so without the
//! refresh a one-node commit hides every other node that holds an artifact.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::facade::core::refresh_retained_diagnostics_views;
use crate::facade::*;
use crate::tests::support::*;
use worth_foundational::ObservationActivationProfile;

type CallLog = Arc<Mutex<BTreeMap<NodeId, u64>>>;

/// `source -> consumer`, plus `bystander`, a source nothing depends on.
struct World {
    runtime: SignalRuntime<(), (), (), (), ()>,
    source: NodeId,
    consumer: NodeId,
    bystander: NodeId,
    calls: CallLog,
}

fn build() -> World {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    let source = graph.node().produces_aspects(mask_a()).build();
    let consumer = graph.node().produces_aspects(mask_a()).on_demand().build();
    graph.append_dependency(consumer, source, ASPECT_A).unwrap();
    let bystander = graph.node().produces_aspects(mask_a()).build();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    // Operational retention with continuous observation: the wasm default
    // (`web_development` + Continuous). History is kept as per-execution
    // reports, which is where the undercount shows.
    runtime.set_runtime_policy(
        SignalRuntimePolicy::operational()
            .with_observation_activation(ObservationActivationProfile::Continuous),
    );
    World {
        runtime,
        source,
        consumer,
        bystander,
        calls: Arc::default(),
    }
}

fn evaluator(
    calls: CallLog,
    consumer: NodeId,
    source: NodeId,
) -> impl Fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError> + Sync {
    move |context| {
        if context.node() == consumer {
            context.read(source, ASPECT_A)?;
        }
        let mut calls = calls.lock().expect("call log mutex poisoned");
        let count = calls.entry(context.node()).or_insert(0);
        *count += 1;
        Ok(context.finish(version_ab(*count, 0)))
    }
}

fn settle(world: &mut World) {
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    world
        .runtime
        .targets([world.consumer, world.bystander])
        .on_demand()
        .read_many(&(), &evaluator)
        .unwrap();
}

#[test]
fn refreshing_retained_views_makes_history_now_describe_the_live_graph() {
    let mut world = build();
    settle(&mut world);
    // All three nodes hold artifacts now. A commit that dirties only the
    // source never touches the bystander, so the latest retained entry
    // traces fewer nodes than hold artifacts.
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    let source = world.source;
    world
        .runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            Ok(())
        })
        .unwrap();
    let before = world.runtime.diagnostics().history_now();
    assert_eq!(
        before.traced_node_count, 1,
        "the commit entry traces the one node it executed, not the bystander"
    );
    let retained_before = world.runtime.diagnostics().recent_history().len();

    refresh_retained_diagnostics_views(world.runtime.graph_mut());

    let after = world.runtime.diagnostics().history_now();
    assert_eq!(after.traced_node_count, 3);
    assert_eq!(
        world.runtime.diagnostics().recent_history().len(),
        retained_before + 1,
        "the refreshed view is one more retained entry, like a snapshot restore"
    );
    // The graph summary is refreshed from the same pass.
    assert_eq!(
        world.runtime.diagnostics().summary_now().active_node_count,
        3
    );
}

#[test]
fn refreshing_an_untouched_graph_traces_nothing() {
    let mut world = build();
    refresh_retained_diagnostics_views(world.runtime.graph_mut());
    assert_eq!(
        world.runtime.diagnostics().history_now().traced_node_count,
        0
    );
    assert_eq!(world.runtime.diagnostics().recent_history().len(), 1);
}
