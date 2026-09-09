use super::*;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::retained_storage::{
    RetainedStorageForkPreparation, RetainedStorageMeasurement,
    SignalConditionalRetentionLedger as Ledger,
    SignalConditionalRetentionReservation as Reservation,
};
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use crate::tests::support::ASPECT_A;

fn graph(population: usize) -> SignalGraph {
    let mut graph = SignalGraph::new();
    let nodes: Vec<_> = (0..=population).map(|_| graph.create_node()).collect();
    for node in &nodes[1..] {
        graph
            .set_dependencies(
                *node,
                [DependencyEdge::whole_partition(
                    nodes[0],
                    ASPECT_A,
                    "partition".repeat(population),
                )],
            )
            .unwrap();
    }
    graph
}
fn ledger(bytes: u64) -> std::sync::Arc<Ledger> {
    Ledger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 1,
            maximum_retained_bytes: bytes,
            maximum_attempt_visits: 1_000_000,
        },
        crate::runtime_policy::SignalConditionalTemporalBudget {
            maximum_live_partitions: 1,
            maximum_reserved_active_wakes: 1,
        },
    )
}

#[test]
fn topology_conversion_growth_is_bounded_and_matches_cold_storage_evidence() {
    let mut profiles = Vec::new();
    for population in [1, 8, 64] {
        let mut graph = graph(population);
        graph
            .topology
            .prepare_fork_charge(&mut Work::new(1_000_000))
            .unwrap();
        let before = graph
            .topology
            .retained_heap_charge(&mut Work::new(1_000_000))
            .unwrap();
        let mut work = Work::new(1_000_000);
        let growth = graph.topology.prepare_fork_growth(&mut work).unwrap();
        profiles.push((growth, work.visits()));
        assert!(graph
            .topology
            .prepare_fork_growth(&mut Work::new(work.visits() - 1))
            .is_err());
        let denied = ledger(growth.bytes() + std::mem::size_of::<Reservation>() as u64 - 1);
        assert!(denied.reserve(0, growth).is_err());
        assert_eq!(
            graph
                .topology
                .retained_heap_charge(&mut Work::new(1_000_000))
                .unwrap(),
            before
        );
        let ledger = ledger(growth.bytes() + std::mem::size_of::<Reservation>() as u64);
        let mut resources = ledger.reserve(0, growth).unwrap();
        let candidate = graph.topology.fork_reserved(&mut resources);
        drop(resources);
        let after = graph
            .topology
            .retained_heap_charge(&mut Work::new(1_000_000))
            .unwrap();
        assert_eq!(after.checked_sub(before).unwrap(), growth);
        assert_eq!(ledger.usage().1, growth.bytes());
        assert_eq!(
            graph
                .topology
                .prepare_fork_growth(&mut Work::new(100_000))
                .unwrap(),
            Charge::ZERO
        );
        drop(graph);
        assert_eq!(ledger.usage().1, growth.bytes());
        drop(candidate);
        assert_eq!(ledger.usage(), (0, 0));
    }
    assert!(profiles.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn evaluation_root_reserved_fork_preserves_payload_and_conversion_custody() {
    let mut graph = graph(8);
    let basis = crate::data::graph::storage::execution_basis::SignalExecutionBasis::capture(
        &mut graph,
        &mut Work::new(1_000_000),
    )
    .unwrap();
    let mut source = basis.evaluation;
    source
        .prepare_mutable_heap_charge(&mut Work::new(1_000_000))
        .unwrap();
    let before = source.diagnostics_for_test();
    let growth = source
        .prepare_fork_growth(&mut Work::new(1_000_000))
        .unwrap();
    let ledger = ledger(growth.bytes() + std::mem::size_of::<Reservation>() as u64);
    let mut resources = ledger.reserve(0, growth).unwrap();
    let candidate = source.fork_reserved(&mut resources);
    drop(resources);
    assert_eq!(candidate.diagnostics_for_test(), before);
    assert!(source.hot.shares_storage_with(&candidate.hot));
    assert!(source.warm.shares_storage_with(&candidate.warm));
    assert!(source.cold.shares_storage_with(&candidate.cold));
    drop(candidate);
    assert_eq!(ledger.usage().1, growth.bytes());
    drop(source);
    assert_eq!(ledger.usage(), (0, 0));
}
