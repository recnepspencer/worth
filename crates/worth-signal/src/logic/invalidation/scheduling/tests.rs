use std::sync::atomic::{AtomicUsize, Ordering};

use crate::data::telemetry::InvalidationPerformedCounter;
use crate::facade::{mark_dirty, EvaluationRequestMode, SignalGraph};
use crate::tests::support::{version_ab, ASPECT_A};

use super::{
    admit_current_readiness, execute_ready, lower_current_work, ReadyInvalidationQueue,
    ReadyQueueEntry,
};

#[test]
fn same_epoch_duplicate_work_enqueues_pops_and_evaluates_once() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let bootstrap = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let before = graph.telemetry().invalidation;
    let epoch = graph.begin_invalidation_readiness_epoch();
    let order = crate::data::proof::invalidation::progression::InvalidationStageOrder {
        stage: 0,
        order: 0,
    };
    let input = match graph.node_invalidation_input(source).unwrap() {
        crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(input) => {
            input
        }
        other => panic!("expected resolved source invalidation, got {other:?}"),
    };
    let first = admit_current_readiness(
        &graph,
        lower_current_work(&graph, source, input.clone(), epoch, order).unwrap(),
        epoch,
        order,
    )
    .unwrap();
    let duplicate = admit_current_readiness(
        &graph,
        lower_current_work(&graph, source, input, epoch, order).unwrap(),
        epoch,
        order,
    )
    .unwrap();
    let mut queue = ReadyInvalidationQueue::new();
    assert!(queue
        .insert(
            &mut graph,
            ReadyQueueEntry {
                task_index: 0,
                ready: first,
            },
        )
        .unwrap());
    assert!(!queue
        .insert(
            &mut graph,
            ReadyQueueEntry {
                task_index: 1,
                ready: duplicate,
            },
        )
        .unwrap());
    let evaluations = AtomicUsize::new(0);
    let ready = queue.pop(&mut graph).unwrap().unwrap().ready;
    execute_ready(&graph, ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(queue.pop(&mut graph).unwrap().is_none());
    let after = graph.telemetry().invalidation;

    assert_eq!(evaluations.load(Ordering::SeqCst), 1);
    assert_eq!(after.ready_items_enqueued - before.ready_items_enqueued, 1);
    assert_eq!(after.ready_items_popped - before.ready_items_popped, 1);
    assert_eq!(
        after.ready_work_deduplicated - before.ready_work_deduplicated,
        1
    );
    assert_eq!(after.retained_ready_frontier_width, 0);
}

#[test]
fn stale_readiness_epoch_cannot_be_reintroduced_by_the_caller() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let bootstrap = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();

    let stale_epoch = graph.begin_invalidation_readiness_epoch();
    let order = crate::data::proof::invalidation::progression::InvalidationStageOrder {
        stage: 0,
        order: 0,
    };
    let input = match graph.node_invalidation_input(source).unwrap() {
        crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(input) => {
            input
        }
        other => panic!("expected resolved source invalidation, got {other:?}"),
    };
    let lowered = lower_current_work(&graph, source, input, stale_epoch, order).unwrap();
    let _current_epoch = graph.begin_invalidation_readiness_epoch();

    let error = admit_current_readiness(&graph, lowered, stale_epoch, order)
        .expect_err("an old epoch must not become current through its own stale binding");
    assert!(error.to_string().contains("stale readiness epoch"));
}

#[test]
fn ready_work_cannot_execute_after_its_readiness_epoch_is_superseded() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let bootstrap = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let ready = current_ready(&mut graph, source);
    let _superseding_epoch = graph.begin_invalidation_readiness_epoch();
    let evaluations = AtomicUsize::new(0);

    let error = execute_ready(&graph, ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect_err("a superseded readiness epoch must fail before execution");
    assert!(error.to_string().contains("stale readiness epoch"));
    assert_eq!(evaluations.load(Ordering::SeqCst), 0);
}

#[test]
fn ready_work_cannot_execute_after_its_source_generation_is_superseded() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let bootstrap = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let ready = current_ready(&mut graph, source);
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let evaluations = AtomicUsize::new(0);

    let error = execute_ready(&graph, ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect_err("a superseded source generation must fail before execution");
    assert!(error.to_string().contains("stale causal authority"));
    assert_eq!(evaluations.load(Ordering::SeqCst), 0);
}

#[test]
fn same_shaped_rewire_invalidates_ready_work_before_execution() {
    let (mut graph, old_source, new_source, consumer) = dependency_graph();
    publish_source_change(&mut graph, old_source, consumer);
    let ready = current_ready(&mut graph, consumer);

    graph
        .rewire_simple_dependency_edge(consumer, old_source, new_source, ASPECT_A)
        .unwrap();
    let evaluations = AtomicUsize::new(0);
    let error = execute_ready(&graph, ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .expect_err("rewiring must invalidate an already-ready batch");

    assert!(error.to_string().contains("stale dependency revision"));
    assert_eq!(evaluations.load(Ordering::SeqCst), 0);
}

#[test]
fn component_access_preserves_source_and_dependency_invalidation_authority() {
    let (mut graph, old_source, new_source, consumer) = dependency_graph();
    mark_dirty(&mut graph, new_source, ASPECT_A).unwrap();
    let source_revision = graph.dependency_revision(new_source).unwrap();
    let source_input = graph.node_invalidation_input(new_source).unwrap();
    let crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(
        source_causes,
    ) = source_input
    else {
        panic!("a direct dirty source must resolve from its source basis");
    };
    assert!(source_causes.is_source_recompute());
    assert!(source_causes.is_bound_to_revision(source_revision));
    assert_eq!(
        source_causes.dirty_aspects(),
        graph.node_dirty_aspects(new_source).unwrap()
    );
    let _source_ready = current_ready(&mut graph, new_source);

    let consumer_revision = graph.dependency_revision(consumer).unwrap();
    publish_source_change(&mut graph, old_source, consumer);
    let pending = graph.pending_causes(consumer).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].key.consumer, consumer);
    assert_eq!(pending[0].key.producer, old_source);
    assert_eq!(pending[0].key.aspect, ASPECT_A);
    assert_eq!(pending[0].key.dependency_revision, consumer_revision);
    let input = graph.node_invalidation_input(consumer).unwrap();
    let crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(causes) =
        input
    else {
        panic!("a published dependency change must resolve from its retained cause");
    };
    assert!(causes.is_bound_to_revision(consumer_revision));
    assert_eq!(
        causes.dirty_aspects(),
        graph.node_dirty_aspects(consumer).unwrap()
    );
    let _consumer_ready = current_ready(&mut graph, consumer);
}

#[test]
fn checkpoint_restore_rejects_old_ready_and_rebuilds_from_current_causes() {
    let (mut graph, old_source, _new_source, consumer) = dependency_graph();
    publish_source_change(&mut graph, old_source, consumer);
    let pre_restore_ready = current_ready(&mut graph, consumer);
    let causes_before = graph.pending_causes(consumer).unwrap().to_vec();
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };
    let mut restored = SignalGraph::restore_from_checkpoint_image(&image).unwrap();

    let error = execute_ready(&restored, pre_restore_ready, || Ok(()))
        .expect_err("a ready batch from the pre-restore graph must not execute");
    assert!(error.to_string().contains("stale graph instance"));
    let causes_after = restored.pending_causes(consumer).unwrap();
    assert_eq!(causes_after.len(), causes_before.len());
    for (after, before) in causes_after.iter().zip(&causes_before) {
        assert_eq!(after.key.consumer, before.key.consumer);
        assert_eq!(
            after.key.dependency_revision,
            before.key.dependency_revision
        );
        assert_eq!(after.key.producer, before.key.producer);
        assert_eq!(after.key.aspect, before.key.aspect);
        assert_eq!(after.key.edge_scope, before.key.edge_scope);
        assert_eq!(
            after.binding_axes.output_commit_ordinal,
            before.binding_axes.output_commit_ordinal
        );
        assert_eq!(
            after.binding_axes.committed_version,
            before.binding_axes.committed_version
        );
        assert_eq!(after.changed_scopes, before.changed_scopes);
    }

    let rebuilt = current_ready(&mut restored, consumer);
    let evaluations = AtomicUsize::new(0);
    execute_ready(&restored, rebuilt, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert_eq!(evaluations.load(Ordering::SeqCst), 1);
}

#[test]
fn rejected_cycle_is_atomic_and_does_not_invalidate_current_ready_work() {
    let (mut graph, source, _new_source, consumer) = dependency_graph();
    publish_source_change(&mut graph, source, consumer);
    let ready = current_ready(&mut graph, consumer);
    let source_revision = graph.dependency_revision(source).unwrap();
    let consumer_revision = graph.dependency_revision(consumer).unwrap();
    let causes = graph.pending_causes(consumer).unwrap().to_vec();
    let performed = graph.invalidation_performed_counters();

    let cycle = graph.set_dependencies(
        source,
        [crate::data::dependency::DependencyEdge::new(
            consumer, ASPECT_A,
        )],
    );
    assert!(cycle.is_err());
    assert_eq!(graph.dependency_revision(source).unwrap(), source_revision);
    assert_eq!(
        graph.dependency_revision(consumer).unwrap(),
        consumer_revision
    );
    assert_eq!(graph.pending_causes(consumer).unwrap(), causes);
    let mut expected_after_rejection = performed.values();
    expected_after_rejection[InvalidationPerformedCounter::RejectedTopologyMutations as usize] += 1;
    assert_eq!(
        graph.invalidation_performed_counters(),
        crate::data::telemetry::SignalInvalidationRealizedCounters::from_values(
            expected_after_rejection,
        )
    );

    let evaluations = AtomicUsize::new(0);
    execute_ready(&graph, ready, || {
        evaluations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert_eq!(evaluations.load(Ordering::SeqCst), 1);
}

fn dependency_graph() -> (
    SignalGraph,
    crate::data::handle::NodeId,
    crate::data::handle::NodeId,
    crate::data::handle::NodeId,
) {
    let mut graph = SignalGraph::new();
    let old_source = graph.node().build();
    let new_source = graph.node().build();
    let consumer = graph.node().build();
    for source in [old_source, new_source] {
        let plan = graph
            .build_evaluation_plan(&[source], EvaluationRequestMode::ForceOnDemand)
            .unwrap();
        graph
            .execute_prepared_plan(&plan, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
            .unwrap();
    }
    graph
        .append_simple_dependency_edge(consumer, old_source, ASPECT_A)
        .unwrap();
    let plan = graph
        .build_evaluation_plan(&[consumer], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|ctx| {
            let _ = ctx.read_aspect_version(old_source, ASPECT_A)?;
            Ok(ctx.finish(version_ab(1, 0)))
        })
        .unwrap();
    (graph, old_source, new_source, consumer)
}

fn publish_source_change(
    graph: &mut SignalGraph,
    source: crate::data::handle::NodeId,
    consumer: crate::data::handle::NodeId,
) {
    mark_dirty(&mut *graph, source, ASPECT_A).unwrap();
    let plan = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|ctx| Ok(ctx.finish(version_ab(2, 0))))
        .unwrap();
    assert!(matches!(
        graph.get_state(consumer).unwrap(),
        crate::data::node::NodeState::Dirty
    ));
}

fn current_ready(
    graph: &mut SignalGraph,
    target: crate::data::handle::NodeId,
) -> crate::data::proof::invalidation::progression::ReadyInvalidationBatch {
    let epoch = graph.begin_invalidation_readiness_epoch();
    let order = crate::data::proof::invalidation::progression::InvalidationStageOrder {
        stage: 0,
        order: 0,
    };
    let input = match graph.node_invalidation_input(target).unwrap() {
        crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(input) => {
            input
        }
        other => panic!("expected resolved invalidation, got {other:?}"),
    };
    let lowered = lower_current_work(graph, target, input, epoch, order).unwrap();
    admit_current_readiness(graph, lowered, epoch, order).unwrap()
}
