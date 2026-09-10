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

#[test]
fn execution_basis_preparation_allocates_no_retained_roots() {
    const CHILD: &str = "WORTH_SIGNAL_CAPTURE_PREPARATION_ALLOCATION_CHILD";
    const TEST: &str = "data::graph::storage::execution_basis::capture_preparation::tests::execution_basis_preparation_allocates_no_retained_roots";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(TEST) && stdout.contains("1 passed; 0 failed"));
        return;
    }
    let (mut graph, _, _) = populated_graph();
    let ledger = crate::data::retained_storage::SignalConditionalRetentionLedger::new(
        crate::runtime_policy::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: u64::MAX,
            maximum_attempt_visits: 100_000,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    for _ in 0..2 {
        let mut work = Preparation::new(100_000);
        let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
        let prepared = SignalExecutionBasis::prepare_capture(&mut graph, &mut work).unwrap();
        let preparation = region.change();
        assert_eq!(preparation.allocations, 0);
        assert_eq!(preparation.reallocations, 0);
        let mut resources = ledger.reserve(0, prepared.charges().source_growth).unwrap();
        let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
        let basis = prepared.capture(&mut resources);
        assert!(
            region.change().allocations > 0,
            "construction is a real allocation control"
        );
        drop(basis);
    }
}
