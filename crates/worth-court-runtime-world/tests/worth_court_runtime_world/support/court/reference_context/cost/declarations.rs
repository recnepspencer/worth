

#[allow(unused_imports)]
pub(crate) use {crate::court::growth_context::cost::profiles};



pub(crate) use {worth_runtime_world::facade::*};
pub(crate) use {crate::court::reference_context::cost::population::Axis, crate::court::reference_context::cost::population::Population};
pub(crate) use {std::time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProbeWork {
    pub(crate) history_lookups: u64,
    pub(crate) pin_contacts: u64,
    pub(crate) pin_hits: u64,
    pub(crate) dependency_acquires: u64,
}
pub(crate) fn observe_probe(population: &Population) -> (u128, ProbeWork) {
    let inspection = population.court.world.inspection_port();
    let before = inspection.retention_costs().unwrap();
    let history = inspection.history_snapshot().unwrap().costs();
    let start = std::time::Instant::now();
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
pub(crate) fn publication_probe(
    population: &mut Population,
) -> (u128, CompositePublicationCostCounters) {
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
    let start = std::time::Instant::now();
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