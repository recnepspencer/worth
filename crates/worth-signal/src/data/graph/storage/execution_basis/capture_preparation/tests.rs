use super::*;
use crate::data::dependency::DependencyEdge;
use crate::facade::{NodeEvaluationResult, SignalRuntimePolicy};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

fn populated_graph() -> (
    SignalGraph,
    crate::data::handle::NodeId,
    crate::data::handle::NodeId,
) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let producer = graph.node().output_identity().build();
    let consumer = graph.node().build();
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::whole_partition(
                producer,
                ASPECT_A,
                "scope".repeat(2_048),
            )],
        )
        .unwrap();
    evaluate(&mut graph, producer, &mut |_, _| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("retained artifact")
            .with_label("retained diagnostic payload"))
    })
    .unwrap();
    (graph, producer, consumer)
}

fn prove_both_fork_charges<T: RetainedStorageMeasurement + RetainedStorageForkPreparation>(
    source: &mut T,
    fork: impl Fn(&mut T) -> T,
) {
    // Independent cold measurements inspect the actual representations. The
    // second iteration exercises already-shared bases. Separate mutation
    // coverage below supplies nonempty overlays through native graph writes.
    for _ in 0..2 {
        let before = source
            .retained_heap_charge(&mut Preparation::new(100_000))
            .unwrap();
        let expected = source
            .prepare_fork_charge(&mut Preparation::new(100_000))
            .unwrap();
        assert_eq!(
            source
                .retained_heap_charge(&mut Preparation::new(100_000))
                .unwrap(),
            before,
            "preparation must not perform conversion"
        );
        let retained = fork(source);
        assert_eq!(
            source
                .retained_heap_charge(&mut Preparation::new(100_000))
                .unwrap(),
            before.checked_add(expected.source_growth).unwrap()
        );
        assert_eq!(
            retained
                .retained_heap_charge(&mut Preparation::new(100_000))
                .unwrap(),
            expected.retained
        );
    }
}

#[test]
fn execution_basis_prospective_roots_charge_source_conversion_and_returned_storage() {
    let (mut graph, _, _) = populated_graph();
    prove_both_fork_charges(&mut graph.arena.definitions, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.nodes, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.free_list, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.free_slots, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.hot, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.warm, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.cold, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.topology, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.cause_sets, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.observation.partition_interner, |root| {
        root.fork_persistent()
    });
}

#[test]
fn execution_basis_capture_consumes_prospective_charge_without_another_preparation() {
    let (mut graph, _, _) = populated_graph();
    for first_capture in [true, false] {
        let mut work = Preparation::new(100_000);
        let prepared = SignalExecutionBasis::prepare_capture(&mut graph, &mut work).unwrap();
        let charges = prepared.charges();
        assert_eq!(charges.source_growth.bytes() > 0, first_capture);
        let visits = work.visits();
        let basis = prepared.capture_for_test();
        assert_eq!(work.visits(), visits);
        assert_eq!(basis.retained_storage_charge(), charges.retained);
        assert_eq!(
            basis
                .measure_seed_charge(&mut Preparation::new(100_000))
                .unwrap(),
            charges.retained
        );
    }
}

#[test]
fn execution_basis_prospective_carrier_cost_ignores_source_history() {
    for graph in [SignalGraph::new(), populated_graph().0] {
        let source = &graph.observation.diagnostics;
        let mut work = Preparation::new(100_000);
        let predicted = source.prepare_branch_carrier_charge(&mut work).unwrap();
        assert!(source
            .prepare_branch_carrier_charge(&mut Preparation::new(work.visits() - 1))
            .is_err());
        let carrier = source.fork_branch_carrier();
        assert_eq!(
            carrier
                .retained_branch_carrier_charge(&mut Preparation::new(100_000))
                .unwrap(),
            predicted
        );
    }
}

#[test]
fn execution_basis_prospective_charge_covers_native_mutations_of_shared_roots() {
    let (mut graph, producer, consumer) = populated_graph();
    let retained =
        SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
    let alternate = graph.node().output_identity().build();
    // Rewiring appends a segment and removes the inherited reverse membership.
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::whole_partition(
                alternate,
                ASPECT_A,
                "new scope".repeat(1_024),
            )],
        )
        .unwrap();
    evaluate(&mut graph, alternate, &mut |_, _| {
        Ok(NodeEvaluationResult::from_version(version_ab(7, 0))
            .with_output_identity("new overlay artifact"))
    })
    .unwrap();
    assert_ne!(
        graph.dependencies_of(consumer).unwrap()[0].source(),
        producer
    );
    prove_both_fork_charges(&mut graph.topology, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.hot, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.arena.cold, |root| root.fork_persistent());
    prove_both_fork_charges(&mut graph.cause_sets, |root| root.fork_persistent());
    let basis = SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
    assert_eq!(
        basis
            .measure_seed_charge(&mut Preparation::new(100_000))
            .unwrap(),
        basis.retained_storage_charge()
    );
    assert!(retained.retained_storage_charge().bytes() > 0);
}
