//! One transaction is one flow. `latest_flow()` after a commit describes the
//! change the transaction made, the invalidation it caused, and every
//! evaluation it ran for that change, however many executions it took
//! (`evaluate_dirty`, then the demand pass, then reads inside the
//! transaction). A clean read after the commit leaves the flow alone, and
//! the next transaction starts a fresh one.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::facade::*;
use crate::tests::support::*;

type CallLog = Arc<Mutex<BTreeMap<NodeId, u64>>>;

/// `source -> consumer`, the consumer on-demand as every wasm recipe node is.
struct World {
    runtime: SignalRuntime<(), (), (), (), ()>,
    source: NodeId,
    consumer: NodeId,
    calls: CallLog,
}

fn build() -> World {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().produces_aspects(mask_a()).build();
    let consumer = graph
        .node()
        .produces_aspects(mask_a())
        .on_demand()
        .build();
    graph.append_dependency(consumer, source, ASPECT_A).unwrap();
    World {
        runtime: SignalRuntime::builder(graph).with_kernel_defaults().build(),
        source,
        consumer,
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

fn calls_for(calls: &CallLog, node: NodeId) -> u64 {
    calls
        .lock()
        .expect("call log mutex poisoned")
        .get(&node)
        .copied()
        .unwrap_or(0)
}

fn settle(world: &mut World) {
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    world
        .runtime
        .targets([world.consumer])
        .on_demand()
        .read_many(&(), &evaluator)
        .unwrap();
}

/// Marks `source` dirty, evaluates the dirty set, then settles `consumer` as
/// standing demand: the wasm commit path.
fn commit_with_demand(world: &mut World) {
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    let (source, consumer) = (world.source, world.consumer);
    world
        .runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            tx.evaluate_demand(&evaluator, &[consumer])?;
            Ok(())
        })
        .unwrap();
}

struct FlowFacts {
    changed_nodes: Vec<NodeId>,
    changed_aspects: Vec<u8>,
    invalidated_direct_subscribers: u32,
    tasks_executed: u32,
    plan_task_count: u32,
    condition_forced_tasks: u32,
    requested_target_tasks: u32,
}

fn latest_flow_facts(world: &World) -> FlowFacts {
    let diagnostics = world.runtime.diagnostics();
    let flow = diagnostics
        .latest_flow()
        .expect("a committed transaction records a flow");
    FlowFacts {
        changed_nodes: flow.change.changed_nodes.clone(),
        changed_aspects: flow.change.changed_aspects.clone(),
        invalidated_direct_subscribers: flow.invalidation.invalidated_direct_subscribers,
        tasks_executed: flow.apply.report.tasks_executed,
        plan_task_count: flow.planning.plan.task_count,
        condition_forced_tasks: flow
            .planning
            .plan
            .task_reason_counts
            .get(&TaskReason::ConditionForced)
            .copied()
            .unwrap_or(0),
        requested_target_tasks: flow
            .planning
            .plan
            .task_reason_counts
            .get(&TaskReason::RequestedTarget)
            .copied()
            .unwrap_or(0),
    }
}

#[test]
fn a_transaction_with_a_demand_pass_records_one_flow_carrying_its_change() {
    let mut world = build();
    settle(&mut world);
    assert_eq!(calls_for(&world.calls, world.consumer), 1);

    commit_with_demand(&mut world);
    assert_eq!(calls_for(&world.calls, world.source), 2);
    assert_eq!(calls_for(&world.calls, world.consumer), 2);

    let facts = latest_flow_facts(&world);
    // The change input belongs to the transaction, not to whichever
    // execution ran last.
    assert_eq!(facts.changed_nodes, vec![world.source]);
    assert_eq!(facts.changed_aspects, vec![ASPECT_A.id()]);
    assert_eq!(facts.invalidated_direct_subscribers, 1);
    // Both executions are in the one flow: the dirty pass planned the source
    // as a requested target, the demand pass forced the on-demand consumer.
    assert_eq!(facts.tasks_executed, 2);
    assert!(facts.requested_target_tasks >= 1, "dirty pass is in the flow");
    assert!(facts.condition_forced_tasks >= 1, "demand pass is in the flow");
}

#[test]
fn a_clean_read_after_commit_leaves_the_transaction_flow_in_place() {
    let mut world = build();
    settle(&mut world);
    commit_with_demand(&mut world);
    let before = latest_flow_facts(&world);

    settle(&mut world);
    assert_eq!(calls_for(&world.calls, world.consumer), 2, "nothing recomputed");
    let after = latest_flow_facts(&world);
    assert_eq!(after.changed_nodes, before.changed_nodes);
    assert_eq!(after.tasks_executed, before.tasks_executed);
    assert_eq!(after.plan_task_count, before.plan_task_count);
}

#[test]
fn the_next_transaction_starts_a_fresh_flow() {
    let mut world = build();
    settle(&mut world);
    commit_with_demand(&mut world);
    let first = latest_flow_facts(&world);
    commit_with_demand(&mut world);
    let second = latest_flow_facts(&world);

    assert_eq!(second.changed_nodes, vec![world.source]);
    assert_eq!(second.changed_aspects, vec![ASPECT_A.id()]);
    assert_eq!(second.tasks_executed, first.tasks_executed, "no carry-over");
    assert_eq!(second.plan_task_count, first.plan_task_count, "no carry-over");
    assert_eq!(calls_for(&world.calls, world.consumer), 3);
}

#[test]
fn outside_a_transaction_each_execution_is_its_own_flow() {
    // Folding is a transaction property. Without one, the execution that
    // follows a change takes the change as its flow, and the read that then
    // settles the consumer is a separate flow with no change of its own.
    let mut world = build();
    settle(&mut world);
    let source = world.source;
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    crate::facade::core::mark_dirty(world.runtime.graph_mut(), source, ASPECT_A).unwrap();
    world.runtime.evaluate_dirty(&(), &evaluator).unwrap();
    let dirty_pass = latest_flow_facts(&world);
    assert_eq!(dirty_pass.changed_nodes, vec![world.source]);
    assert_eq!(dirty_pass.changed_aspects, vec![ASPECT_A.id()]);
    assert_eq!(dirty_pass.tasks_executed, 1, "the source alone; the consumer is on-demand");

    settle(&mut world);
    assert_eq!(calls_for(&world.calls, world.consumer), 2);
    let read_pass = latest_flow_facts(&world);
    assert_eq!(read_pass.changed_nodes, Vec::<NodeId>::new());
    assert_eq!(read_pass.tasks_executed, 1, "not folded into the dirty pass");
}

#[test]
fn a_rolled_back_transaction_does_not_leak_its_change_into_the_next_flow() {
    let mut world = build();
    settle(&mut world);
    let evaluator = evaluator(world.calls.clone(), world.consumer, world.source);
    let (source, consumer) = (world.source, world.consumer);
    let rolled_back: Result<(), SignalError> = world.runtime.transaction(&mut (), |tx| {
        tx.mark_dirty(source, ASPECT_A)?;
        tx.evaluate_dirty(&evaluator)?;
        tx.evaluate_demand(&evaluator, &[consumer])?;
        Err(SignalError::invalid_input("abandon"))
    }).map(|_| ());
    assert!(rolled_back.is_err());

    commit_with_demand(&mut world);
    let facts = latest_flow_facts(&world);
    assert_eq!(facts.changed_nodes, vec![world.source]);
    assert_eq!(facts.tasks_executed, 2, "only the committed transaction's work");
}
