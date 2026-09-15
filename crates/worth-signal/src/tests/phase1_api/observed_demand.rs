//! Observation is demand: a transaction that invalidates a watched on-demand
//! node must recompute it before commit so per-committed-transaction delivery
//! can classify it. `SignalTransaction::evaluate_observed_demand` is that pass.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::facade::*;
use crate::tests::support::*;

struct RecordingListener {
    notices: Arc<Mutex<Vec<(Vec<NodeId>, bool, bool)>>>,
}

impl ObservationListener<(), (), (), (), ()> for RecordingListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        notice: &ObservationNotice<'_>,
    ) {
        self.notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .push((
                notice.matched_nodes().iter().collect(),
                notice.recomputed(),
                notice.meaningful_change(),
            ));
    }
}

/// `source -> first -> second`, both computeds on-demand, plus an unrelated
/// on-demand computed fed by its own source.
struct Chain {
    source: NodeId,
    first: NodeId,
    second: NodeId,
    unrelated_source: NodeId,
    unrelated: NodeId,
    /// Declared upstreams per node; evaluators re-read them through the
    /// context so an evaluation keeps the topology instead of wiping it.
    upstreams: BTreeMap<NodeId, Vec<NodeId>>,
}

fn build_chain() -> (SignalRuntime<(), (), (), (), ()>, Chain) {
    let mut graph = SignalGraph::new();
    // Every producer declares aspect A so committed output deltas reach its
    // consumers through the reverse subscriber index, as recipes do.
    let source = graph.node().produces_aspects(mask_a()).build();
    let first = graph.node().produces_aspects(mask_a()).on_demand().build();
    let second = graph.node().produces_aspects(mask_a()).on_demand().build();
    let unrelated_source = graph.node().produces_aspects(mask_a()).build();
    let unrelated = graph.node().produces_aspects(mask_a()).on_demand().build();
    graph.append_dependency(first, source, ASPECT_A).unwrap();
    graph.append_dependency(second, first, ASPECT_A).unwrap();
    graph
        .append_dependency(unrelated, unrelated_source, ASPECT_A)
        .unwrap();
    let runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    (
        runtime,
        Chain {
            source,
            first,
            second,
            unrelated_source,
            unrelated,
            upstreams: BTreeMap::from([
                (first, vec![source]),
                (second, vec![first]),
                (unrelated, vec![unrelated_source]),
            ]),
        },
    )
}

type Upstreams = BTreeMap<NodeId, Vec<NodeId>>;

fn read_upstreams(
    context: &mut EvaluationContext<'_, ()>,
    upstreams: &Upstreams,
) -> Result<(), SignalError> {
    for upstream in upstreams.get(&context.node()).into_iter().flatten() {
        context.read(*upstream, ASPECT_A)?;
    }
    Ok(())
}

type CallLog = Arc<Mutex<BTreeMap<NodeId, u64>>>;

/// Every evaluation of a node publishes a new version, so each recompute is a
/// meaningful change; the log records how many times each node ran.
fn changing_evaluator(
    calls: CallLog,
    upstreams: Upstreams,
) -> impl Fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError> + Sync {
    move |context| {
        read_upstreams(context, &upstreams)?;
        let mut calls = calls.lock().expect("evaluator call log mutex poisoned");
        let count = calls.entry(context.node()).or_insert(0);
        *count += 1;
        Ok(context.finish(version_ab(*count, 0)))
    }
}

fn calls_for(calls: &CallLog, node: NodeId) -> u64 {
    calls
        .lock()
        .expect("evaluator call log mutex poisoned")
        .get(&node)
        .copied()
        .unwrap_or(0)
}

/// Sources publish a new version on every evaluation; computeds (nodes with
/// upstreams) always publish the same one, so they are recomputed but never
/// meaningfully changed.
fn constant_computeds_evaluator(
    calls: CallLog,
    upstreams: Upstreams,
) -> impl Fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError> + Sync {
    move |context| {
        read_upstreams(context, &upstreams)?;
        let mut calls = calls.lock().expect("evaluator call log mutex poisoned");
        let count = calls.entry(context.node()).or_insert(0);
        *count += 1;
        let version = if upstreams.contains_key(&context.node()) {
            1
        } else {
            *count
        };
        Ok(context.finish(version_ab(version, 0)))
    }
}

fn settle_on_demand(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    nodes: &[NodeId],
    evaluator: &(impl Fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError> + Sync),
) {
    runtime
        .targets(nodes.iter().copied())
        .on_demand()
        .read_many(&(), evaluator)
        .unwrap();
}

fn node_state(runtime: &SignalRuntime<(), (), (), (), ()>, node: NodeId) -> NodeState {
    runtime.observe().graph().graph().get_state(node).unwrap()
}

#[test]
fn watched_on_demand_chain_is_recomputed_and_delivered_by_the_committing_transaction() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second, chain.unrelated], &evaluator);
    assert_eq!(node_state(&runtime, chain.second), NodeState::Clean);

    let notices = Arc::new(Mutex::new(Vec::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [chain.second],
        Box::new(RecordingListener {
            notices: Arc::clone(&notices),
        }),
    );
    let before = *runtime.telemetry();

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_observed_demand(&evaluator)?);
            Ok(())
        })
        .unwrap();

    let summary = summary.expect("transaction closure ran");
    assert_eq!(
        summary,
        ObservedDemandSummary {
            // source, first, second: the whole downstream reach of the change.
            reach_visits: 3,
            targets: 1,
            // Pass one recomputes first then second; pass two finds nothing.
            passes: 2,
            tasks_executed: 2,
        }
    );
    assert_eq!(calls_for(&calls, chain.first), 2);
    assert_eq!(calls_for(&calls, chain.second), 2);
    assert_eq!(
        calls_for(&calls, chain.unrelated),
        1,
        "an unrelated watched-or-not on-demand node is never demanded"
    );
    assert_eq!(node_state(&runtime, chain.second), NodeState::Clean);

    let notices = notices.lock().expect("observed demand notices mutex poisoned");
    assert_eq!(
        notices.as_slice(),
        &[(vec![chain.second], true, true)],
        "the watched on-demand node is delivered once, recomputed and meaningfully changed"
    );

    let after = *runtime.telemetry();
    assert_eq!(after.transaction.observed_demand_reach_visits - before.transaction.observed_demand_reach_visits, 3);
    assert_eq!(after.transaction.observed_demand_targets - before.transaction.observed_demand_targets, 1);
    assert_eq!(after.transaction.observed_demand_passes - before.transaction.observed_demand_passes, 2);
}

#[test]
fn without_the_demand_pass_a_watched_on_demand_node_is_never_delivered() {
    // The regression this pass exists for: everything else identical, the
    // watcher of an on-demand node hears nothing and the node stays stale.
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second], &evaluator);

    let notices = Arc::new(Mutex::new(Vec::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [chain.second],
        Box::new(RecordingListener {
            notices: Arc::clone(&notices),
        }),
    );
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            Ok(())
        })
        .unwrap();

    assert_eq!(calls_for(&calls, chain.first), 1);
    assert_eq!(calls_for(&calls, chain.second), 1);
    assert_eq!(node_state(&runtime, chain.first), NodeState::Dirty);
    assert!(
        notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .is_empty()
    );
}

#[test]
fn unrelated_watched_on_demand_nodes_are_not_demanded() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());

    let notices = Arc::new(Mutex::new(Vec::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [chain.unrelated],
        Box::new(RecordingListener {
            notices: Arc::clone(&notices),
        }),
    );

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_observed_demand(&evaluator)?);
            Ok(())
        })
        .unwrap();

    assert_eq!(
        summary.unwrap(),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 0,
            passes: 0,
            tasks_executed: 0,
        }
    );
    assert_eq!(calls_for(&calls, chain.unrelated), 0);
    assert_ne!(node_state(&runtime, chain.unrelated), NodeState::Clean);
    assert!(
        notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .is_empty()
    );

    // Touching its own source is what demands it.
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.unrelated_source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_observed_demand(&evaluator)?);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        summary.unwrap(),
        ObservedDemandSummary {
            reach_visits: 2,
            targets: 1,
            passes: 2,
            tasks_executed: 1,
        }
    );
    assert_eq!(calls_for(&calls, chain.unrelated), 1);
    assert_eq!(
        notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .len(),
        1
    );
}

#[test]
fn demanded_recompute_that_does_not_change_is_not_a_meaningful_change() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let constant = constant_computeds_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second], &constant);

    let notices = Arc::new(Mutex::new(Vec::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [chain.second],
        Box::new(RecordingListener {
            notices: Arc::clone(&notices),
        }),
    );
    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&constant)?;
            summary = Some(tx.evaluate_observed_demand(&constant)?);
            Ok(())
        })
        .unwrap();

    // `first` is recomputed because the source changed; its output is
    // identical, so `second` is never invalidated and never runs.
    assert_eq!(
        summary.unwrap(),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 1,
            passes: 2,
            tasks_executed: 1,
        }
    );
    assert_eq!(calls_for(&calls, chain.first), 2);
    assert_eq!(calls_for(&calls, chain.second), 1);
    assert!(
        notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .is_empty(),
        "meaningful-change watchers stay quiet when the demanded value is unchanged"
    );
}

#[test]
fn demand_pass_is_free_when_nothing_is_observed() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_observed_demand(&evaluator)?);
            Ok(())
        })
        .unwrap();
    assert_eq!(summary.unwrap(), ObservedDemandSummary::default());
    assert_eq!(calls_for(&calls, chain.first), 0);
}

#[test]
fn standing_demand_settles_a_reached_node_without_any_observer() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second, chain.unrelated], &evaluator);
    let before = *runtime.telemetry();

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_demand(&evaluator, &[chain.second, chain.unrelated])?);
            Ok(())
        })
        .unwrap();

    // Reach is source, first, second. `unrelated` is standing demand but not
    // reached by this change, so it is not a target and never recomputed.
    assert_eq!(
        summary.expect("transaction closure ran"),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 1,
            passes: 2,
            tasks_executed: 2,
        }
    );
    assert_eq!(node_state(&runtime, chain.first), NodeState::Clean);
    assert_eq!(node_state(&runtime, chain.second), NodeState::Clean);
    assert_eq!(calls_for(&calls, chain.first), 2);
    assert_eq!(calls_for(&calls, chain.second), 2);
    assert_eq!(calls_for(&calls, chain.unrelated), 1);
    let after = *runtime.telemetry();
    assert_eq!(
        after.transaction.observed_demand_targets - before.transaction.observed_demand_targets,
        1
    );
}

#[test]
fn standing_demand_that_the_change_does_not_reach_is_not_demanded() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second, chain.unrelated], &evaluator);

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_demand(&evaluator, &[chain.unrelated])?);
            Ok(())
        })
        .unwrap();

    assert_eq!(
        summary.expect("transaction closure ran"),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 0,
            passes: 0,
            tasks_executed: 0,
        }
    );
    // Nothing demanded `first`, so it stays deferred like any on-demand node.
    assert_eq!(node_state(&runtime, chain.first), NodeState::Dirty);
    assert_eq!(calls_for(&calls, chain.first), 1);
    assert_eq!(calls_for(&calls, chain.unrelated), 1);
}
