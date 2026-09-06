mod population;
mod profiles;
mod writers;
use super::*;
use population::{Axis, Population};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbeWork {
    history_lookups: u64,
    pin_contacts: u64,
    pin_hits: u64,
    dependency_acquires: u64,
}
fn observe_probe(population: &Population) -> (u128, ProbeWork) {
    let inspection = population.court.world.inspection_port();
    let before = inspection.retention_costs().unwrap();
    let history = inspection.history_snapshot().unwrap().costs();
    let start = Instant::now();
    let observed = population.court.observe(&population.root);
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(
        observed.selected_commit(),
        population.root.selected_commit()
    );
    drop(observed);
    let after = inspection.retention_costs().unwrap();
    let next_history = inspection.history_snapshot().unwrap().costs();
    let work = ProbeWork {
        history_lookups: next_history.entry_lookups() - history.entry_lookups(),
        pin_contacts: after.owner_acquisition_contacts() - before.owner_acquisition_contacts(),
        pin_hits: after.unique_pin_hits() - before.unique_pin_hits(),
        dependency_acquires: after.dependency_acquires() - before.dependency_acquires(),
    };
    assert_eq!(
        work.pin_contacts, 0,
        "admitted observation reuses existing exact pins"
    );
    let report = population
        .court
        .world
        .lifecycle_port()
        .reclaim_retention(
            &[RuntimeWorldRetentionKey::signal(population.root.basis())],
            1,
        )
        .unwrap();
    assert!(report.examined() <= 1);
    assert_eq!(report.reclaimed(), 0, "reachable root cannot be pruned");
    (elapsed, work)
}
fn publication_probe(population: &mut Population) -> (u128, CompositePublicationCostCounters) {
    let before = population
        .court
        .world
        .inspection_port()
        .retention_costs()
        .unwrap();
    let history_before = population
        .court
        .world
        .inspection_port()
        .history_snapshot()
        .unwrap()
        .costs();
    let start = Instant::now();
    let done = population.court.publish_cargo(&population.root, "9");
    let elapsed = start.elapsed().as_nanos();
    let costs = done.cost_counters();
    population.root = population.court.observe(&population.root);
    assert_eq!(costs.relational_owner_contacts(), 1);
    assert_eq!(costs.signal_owner_contacts(), 0);
    assert_eq!(costs.cas_wins(), 1);
    let history_after = population
        .court
        .world
        .inspection_port()
        .history_snapshot()
        .unwrap()
        .costs();
    assert_eq!(
        history_after.reserved_entry_writes() - history_before.reserved_entry_writes(),
        1,
        "publication writes one carried history slot at every population size"
    );
    let after = population
        .court
        .world
        .inspection_port()
        .retention_costs()
        .unwrap();
    assert_eq!(
        after.signal_contacts(),
        before.signal_contacts(),
        "publication reuses the exact Signal pin"
    );
    drop(done);
    (elapsed, costs)
}
#[test]
fn court_structural_costs_do_not_scan_unrelated_b_h_u_a_p_o_populations() {
    let mut baseline = None;
    let mut publication_baseline = None;
    for axis in Axis::ALL {
        for size in [1, 4, 8] {
            let mut population = Population::build(axis, size);
            let (_, work) = observe_probe(&population);
            assert_eq!(
                *baseline.get_or_insert(work),
                work,
                "axis={axis:?}, size={size}"
            );
            let (_, costs) = publication_probe(&mut population);
            assert_eq!(*publication_baseline.get_or_insert(costs), costs);
            population.finish();
        }
    }
}
#[test]
fn court_concurrent_writers_keep_per_attempt_costs_constant() {
    let mut baseline = None;
    for width in [1, 4, 8] {
        let (_, costs) = writers::run(width);
        assert_eq!(*baseline.get_or_insert(costs), costs);
    }
}
