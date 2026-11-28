#[allow(unused_imports)]
pub(crate) use crate::cost::support::growth_context::profiles;

pub(crate) use crate::cost::support::reference_context::population::Population;
pub(crate) use worth_runtime_world::facade::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProbeWork {
    pub(crate) history_lookups: u64,
    pub(crate) pin_contacts: u64,
}
pub(crate) fn observe_probe(population: &mut Population) -> (u128, ProbeWork) {
    let inspection = population.court.world.inspection_port();
    let signal = population
        .court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let relational = &population.court.records.runtime;
    let before_relational = relational.branch_basis_cost_counters();
    let before_signal = signal.owner_service_cost_snapshot().unwrap();
    let history = inspection.history_snapshot().unwrap().costs();
    let start = std::time::Instant::now();
    let observed = population.court.observe(&population.root);
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(
        observed.selected_commit(),
        population.root.selected_commit()
    );
    drop(observed);
    let after_relational = relational.branch_basis_cost_counters();
    let after_signal = signal.owner_service_cost_snapshot().unwrap();
    let next_history = inspection.history_snapshot().unwrap().costs();
    let work = ProbeWork {
        history_lookups: next_history.entry_lookups() - history.entry_lookups(),
        pin_contacts: (after_relational.external_retention_acquires
            - before_relational.external_retention_acquires)
            + (after_signal.retention_registry_contacts()
                - before_signal.retention_registry_contacts()),
    };
    assert_eq!(
        work.pin_contacts, 0,
        "admitted observation reuses existing exact pins"
    );
    (elapsed, work)
}
pub(crate) fn publication_probe(
    population: &mut Population,
) -> (u128, CompositePublicationCostCounters) {
    let signal = population
        .court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .retention_registry_contacts();
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
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .retention_registry_contacts(),
        before,
        "publication reuses the exact Signal pin"
    );
    drop(done);
    (elapsed, costs)
}
