use crate::easy::ReactiveGraph;
use crate::facade::*;
use crate::tests::support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Feature,
}

#[test]
fn locality_footprint_merges_and_detects_conflicts_canonically() {
    let node_a = NodeId::new(7, 1);
    let node_b = NodeId::new(3, 2);
    let node_c = NodeId::new(9, 1);

    let mut left = LocalityFootprint::new(
        vec![
            PartitionSubscription::whole_partition("wing"),
            PartitionSubscription::partition_and_detail("fuselage", "frame-2"),
        ],
        vec![node_a, node_b],
        vec![node_b],
    );
    let right = LocalityFootprint::new(
        vec![
            PartitionSubscription::partition_and_detail("fuselage", "frame-2"),
            PartitionSubscription::whole_partition("tail"),
        ],
        vec![node_b, node_c],
        vec![node_c],
    );

    assert!(left.conflicts_with(&right));
    left.merge(&right);

    assert_eq!(left.partitions.len(), 3);
    assert_eq!(left.nodes.as_slice(), &[node_b, node_a, node_c]);
    assert_eq!(left.sources.as_slice(), &[node_b, node_c]);
}

#[test]
fn graph_node_builder_accepts_explicit_node_contract() {
    let mut graph = SignalGraph::new();
    let contract = NodeContract::reads([ASPECT_A])
        .with_produces([ASPECT_B])
        .with_required_context(ContextRequirement::RelationalSnapshot);
    let node = graph.node().with_contract(contract.clone()).build();

    let stored = graph.get_contract(node).unwrap().clone();
    assert_eq!(stored, contract);
}

#[test]
fn transaction_batch_dirty_is_the_bulk_invalidation_surface() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let dependent = graph.node().build();
    graph
        .set_dependencies(
            dependent,
            [
                DependencyEdge::new(source_a, ASPECT_A),
                DependencyEdge::new(source_b, ASPECT_B),
            ],
        )
        .unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    runtime
        .transaction(request_execution, &mut (), |transaction| {
            transaction.mark_dirty_batch(&DirtyBatch::from_sources([
                (source_a, ASPECT_A),
                (source_b, ASPECT_B),
            ]))?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime.graph().get_state(source_a).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        runtime.graph().get_state(source_b).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        runtime.graph().get_state(dependent).unwrap(),
        NodeState::Dirty
    );
}

#[test]
fn dependency_batch_edit_is_the_bulk_dependency_surface() {
    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let left = graph.node().build();
    let right = graph.node().build();

    graph
        .apply_dependency_batch_edit(&DependencyBatchEdit::from_pairs([
            (left, vec![DependencyEdge::new(source_a, ASPECT_A)]),
            (right, vec![DependencyEdge::new(source_b, ASPECT_B)]),
        ]))
        .unwrap();

    assert_eq!(graph.dependencies_of(left).unwrap().len(), 1);
    assert_eq!(graph.dependencies_of(right).unwrap().len(), 1);
    assert_eq!(graph.runtime_subscribers_of(source_a).unwrap(), &[left]);
    assert_eq!(graph.runtime_subscribers_of(source_b).unwrap(), &[right]);
}

#[test]
#[should_panic(expected = "dependency batch edit cannot contain multiple edits")]
fn dependency_batch_edit_rejects_duplicate_node_edits() {
    let node = NodeId::new(7, 1);
    let source = NodeId::new(3, 2);
    let _ = DependencyBatchEdit::from_pairs([
        (node, vec![DependencyEdge::new(source, ASPECT_A)]),
        (node, vec![DependencyEdge::new(source, ASPECT_B)]),
    ]);
}

#[test]
fn define_computation_applies_contract_comparator_and_tier_to_created_nodes() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_tiers::<Tier>()
        .build();
    let contract = NodeContract::reads([ASPECT_A])
        .with_produces([ASPECT_B])
        .with_required_context(ContextRequirement::DomainContext);
    let computation = runtime
        .define(Recipe {
            family: "geometry".into(),
            contract: contract.clone(),
            tier: Tier::Feature,
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |_ctx: &mut EvaluationContext<'_, ()>| {
                Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(
                    NodeEvaluationResult::from_version(version_ab(1, 0)),
                ))
            },
        })
        .unwrap();

    let node = computation.keyed("bulkhead").node(&mut runtime);
    let stored = runtime
        .graph()
        .get_entry(node)
        .unwrap()
        .get_eval_config()
        .clone();

    assert_eq!(runtime.graph().get_contract(node).unwrap(), &contract);
    assert_eq!(
        stored.comparator,
        Some(VersionComparatorPolicy::OutputIdentity)
    );
    assert_eq!(
        runtime.config().node_meta().tier_for_node(node),
        Some(Tier::Feature)
    );
}

#[test]
fn easy_mode_supports_input_computed_get_set_and_batch() {
    let mut graph = ReactiveGraph::new();
    let price = graph.input(100.0_f64);
    let tax = graph.input(0.08_f64);
    let total = graph.computed(move |context| context.get(price) * (1.0 + context.get(tax)));

    assert_eq!(graph.get(total), 108.0);

    graph.set(price, 200.0);
    assert_eq!(graph.get(total), 216.0);

    graph.batch(|reactive| {
        reactive.set(price, 300.0);
        reactive.set(tax, 0.10);
    });
    assert_eq!(graph.get(total), 330.0);
}

#[test]
fn easy_mode_computed_chains_observe_staged_upstream_values_in_the_same_pass() {
    let mut graph = ReactiveGraph::new();
    let source = graph.input(2_i32);
    let doubled = graph.computed(move |context| context.get(source) * 2);
    let chained = graph.computed(move |context| context.get(doubled) + 1);

    assert_eq!(graph.get(chained), 5);

    graph.set(source, 7);

    assert_eq!(
        graph.get(chained),
        15,
        "downstream computed nodes should see freshly staged upstream values, not the pre-plan cache"
    );
}

#[test]
fn easy_mode_dynamic_dependency_capture_settles_newly_selected_dirty_upstream() {
    let mut graph = ReactiveGraph::new();
    let source = graph.input(1_i32);
    let enabled = graph.input(true);
    let fallback = graph.computed(move |context| context.get(source) * 10);
    let label = graph.computed(move |context| {
        if context.get(enabled) {
            5
        } else {
            context.get(fallback)
        }
    });

    assert_eq!(graph.get(label), 5);
    graph.set(source, 2);
    graph.set(enabled, false);

    assert_eq!(
        graph.get(label),
        20,
        "the first sink read must settle a newly captured dirty dependency"
    );
}

#[test]
fn runtime_sink_read_converges_new_direct_causes_across_a_clean_chain() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let doubled = graph.node().build();
    let sink = graph.node().build();
    graph.append_dependency(doubled, source, ASPECT_A).unwrap();
    graph.append_dependency(sink, doubled, ASPECT_A).unwrap();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let source_value = AtomicU64::new(2);
    let evaluator = |view: &mut EvaluationContext<'_, ()>| {
        let value = if view.node() == source {
            source_value.load(Ordering::Relaxed)
        } else if view.node() == doubled {
            view.read_aspect_version(source, ASPECT_A)?.get(ASPECT_A) * 2
        } else {
            view.read_aspect_version(doubled, ASPECT_A)?.get(ASPECT_A) + 1
        };
        Ok::<EvaluationOutput, SignalError>(
            view.finish(AspectVersion::from_updates([(ASPECT_A, value)])),
        )
    };

    assert_eq!(
        runtime.read(sink, &(), &evaluator).unwrap().get(ASPECT_A),
        5
    );
    source_value.store(7, Ordering::Relaxed);
    mark_dirty(runtime.graph_mut(), source, ASPECT_A).unwrap();

    assert_eq!(
        runtime.read(sink, &(), &evaluator).unwrap().get(ASPECT_A),
        15,
        "one sink read must settle each newly committed direct-hop cause"
    );
}

#[test]
fn easy_mode_failed_batch_restores_input_values() {
    let mut graph = ReactiveGraph::new();
    let price = graph.input(100_i32);
    let tax = graph.input(5_i32);

    let err = graph.try_batch(|reactive| {
        reactive.try_set(price, 200)?;
        reactive.try_set(tax, 9)?;
        Err(SignalError::invalid_input("force easy-mode rollback"))
    });
    assert!(err.is_err());

    assert_eq!(graph.get(price), 100);
    assert_eq!(graph.get(tax), 5);
}

#[test]
fn failed_multi_target_read_restores_every_evaluated_target() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let first = runtime.graph_mut().node().build();
    let second = runtime.graph_mut().node().build();
    let baseline = |view: &mut EvaluationContext<'_, ()>| {
        Ok::<EvaluationOutput, SignalError>(
            view.finish(AspectVersion::from_updates([(ASPECT_A, 1)])),
        )
    };
    runtime.read(first, &(), &baseline).unwrap();
    runtime.read(second, &(), &baseline).unwrap();
    mark_dirty(runtime.graph_mut(), first, ASPECT_A).unwrap();
    mark_dirty(runtime.graph_mut(), second, ASPECT_A).unwrap();

    let error = runtime.transaction(request_execution, &mut (), |tx| {
        tx.read_many(&[first, second], &|view| {
            if view.node() == second {
                return Err(SignalError::invalid_input("fail second batch target"));
            }
            Ok(view.finish(AspectVersion::from_updates([(ASPECT_A, 2)])))
        })?;
        Ok(())
    });

    assert!(error.is_err());
    assert_eq!(
        runtime
            .graph()
            .node_aspect_version(first)
            .unwrap()
            .get(ASPECT_A),
        1
    );
    assert_eq!(
        runtime
            .graph()
            .node_aspect_version(second)
            .unwrap()
            .get(ASPECT_A),
        1
    );
    assert_eq!(runtime.graph().get_state(first).unwrap(), NodeState::Dirty);
    assert_eq!(runtime.graph().get_state(second).unwrap(), NodeState::Dirty);
}
