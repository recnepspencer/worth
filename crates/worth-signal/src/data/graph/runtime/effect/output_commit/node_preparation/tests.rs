//! Native output-boundary evidence; full planner stamping is tested separately.
use super::*;
use crate::data::aspect::AspectVersion;
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::graph::storage::execution_basis::SignalExecutionBasis;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::retained_storage::{RetainedStorageMeasurement, SignalConditionalRetentionLedger};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A};

fn capture(
    graph: &mut SignalGraph,
) -> (
    crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
    std::sync::Arc<SignalConditionalRetentionLedger>,
    crate::data::retained_storage::SignalConditionalRetentionReservation,
    (usize, u64),
) {
    let ledger = SignalConditionalRetentionLedger::new(
        graph
            .installed_runtime_policy()
            .conditional_evaluation_budget(),
        graph
            .installed_runtime_policy()
            .conditional_temporal_budget(),
    );
    let prepared = SignalExecutionBasis::prepare_capture(graph, &mut Work::new(100_000)).unwrap();
    let charges = prepared.charges();
    let mut custody = ledger
        .reserve(
            0,
            charges.retained.checked_add(charges.source_growth).unwrap(),
        )
        .unwrap();
    let basis = prepared.capture(&mut custody);
    let before_partition = ledger.usage();
    (
        basis.new_evaluation_partition(),
        ledger,
        custody,
        before_partition,
    )
}

fn apply(
    graph: &mut SignalGraph,
    producer: NodeId,
    version: u64,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let mut effect = crate::data::graph::runtime::effect::tests::test_effect_with_labels(vec![
        "retained output".into(),
    ]);
    effect.operational.node = producer;
    effect.operational.aspect_version = version_ab(version, 0);
    graph
        .apply_effect(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            &mut DefaultComparatorPolicyResolver::default(),
            false,
            work,
        )
        .map(|_| ())
}

fn assert_carried_charges(graph: &SignalGraph) {
    let carried = [
        graph.arena.hot.prepared_retained_charge(),
        graph.arena.warm.prepared_retained_charge(),
        graph.arena.cold.prepared_retained_charge(),
    ];
    let measured = [
        graph
            .arena
            .hot
            .retained_heap_charge(&mut Work::new(100_000))
            .unwrap(),
        graph
            .arena
            .warm
            .retained_heap_charge(&mut Work::new(100_000))
            .unwrap(),
        graph
            .arena
            .cold
            .retained_heap_charge(&mut Work::new(100_000))
            .unwrap(),
    ];
    assert_eq!(carried, measured.map(Ok));
}

#[test]
fn native_retained_output_installs_cumulative_roots_without_changing_source() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(crate::runtime_policy::SignalRuntimePolicy::forensic());
    let producer = graph.node().produces_aspects(ASPECT_A).build();
    let consumer = graph.node().build();
    graph
        .append_dependency(consumer, producer, ASPECT_A)
        .unwrap();
    evaluate(&mut graph, consumer, &mut |_, _| Ok(AspectVersion::zero())).unwrap();
    let source = graph.node_aspect_version(producer).unwrap();
    let (mut slot, ledger, custody, captured_usage) = capture(&mut graph);
    slot.execute(&mut graph, |selected| {
        apply(selected, producer, 9, &mut EvaluationWork::Ordinary).unwrap();
        assert_carried_charges(selected);
        assert!(!selected.pending_causes(consumer).unwrap().is_empty());
        apply(selected, producer, 10, &mut EvaluationWork::Ordinary).unwrap();
        assert_carried_charges(selected);
        assert_eq!(
            selected.node_aspect_version(producer).unwrap(),
            version_ab(10, 0)
        );
    })
    .unwrap();
    assert_eq!(graph.node_aspect_version(producer).unwrap(), source);
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert!(ledger.usage().1 > captured_usage.1);
    drop(slot);
    assert_eq!(ledger.usage(), captured_usage);
    drop(custody);
    drop(graph);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn closed_owner_denies_native_root_preparation_without_output_movement() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().build();
    let (mut slot, ledger, _custody, _) = capture(&mut graph);
    let usage = ledger.usage();
    ledger.close();
    slot.execute(&mut graph, |selected| {
        let before = selected.node_aspect_version(producer).unwrap();
        assert_eq!(
            apply(selected, producer, 9, &mut EvaluationWork::Ordinary),
            Err(SignalError::EvaluationStorageUnavailable)
        );
        assert_eq!(selected.node_aspect_version(producer).unwrap(), before);
        assert_carried_charges(selected);
    })
    .unwrap();
    assert_eq!(ledger.usage(), usage);
}

#[test]
fn retained_lineage_stamp_consumes_the_callers_remaining_work() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().build();
    let (mut slot, _ledger, _custody, _) = capture(&mut graph);
    slot.execute(&mut graph, |selected| {
        apply(selected, producer, 9, &mut EvaluationWork::Ordinary).unwrap();
        let previous = selected.node_execution_trace_stamp(producer).unwrap();
        let artifact = selected
            .diagnostics_state_mut()
            .allocate_lineage_artifact_id();
        let mut exhausted = Work::new(0);
        let result = selected.stamp_runtime_artifact_lineage_and_execution(
            producer,
            artifact,
            crate::logic::planner::ExecutionRecordId(1),
            crate::logic::planner::SemanticSegmentId(1),
            &mut EvaluationWork::Conditional(&mut exhausted),
        );
        assert_eq!(
            result,
            Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits: 0 })
        );
        assert_eq!(
            selected.node_execution_trace_stamp(producer).unwrap(),
            previous
        );
        assert_carried_charges(selected);
    })
    .unwrap();
}
