//! Each request entry preserves its own ceiling despite unrelated noted debt.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;

#[test]
fn registry_eager_foreign_debt_does_not_refuse_terminal_entry() {
    check(0);
}
#[test]
fn registry_eager_foreign_debt_does_not_refuse_cache_entry() {
    check(1);
}
#[test]
fn registry_eager_foreign_debt_does_not_refuse_joined_entry() {
    check(2);
}

fn check(entry: u8) {
    let context = world::Context::new();
    let mut world = World::new(&context, 1);
    let owner = world
        .registry
        .admit(
            key("oracle-reader", world.generation, 250),
            None,
            context.scope,
            occurrence(),
            super::super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
            &mut record_admission(),
        )
        .map(|(interest, _pause)| interest)
        .unwrap();
    let own_price = match entry {
        0 => 2, // Empty terminal header, with no terminal row to retire.
        1 => 0, // Zero more bytes already fit; there is no cache reclamation.
        2 => world.registry.state.lock().unwrap().records.len() as u64 + 4,
        _ => panic!("entry"),
    };
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(own_price),
        maximum_preparation_bytes: None,
    });
    match entry {
        0 => world
            .registry
            .drain_terminal_cleanup_admitted(&invalidation_owner(), &mut admission)
            .unwrap(),
        1 => world
            .registry
            .reclaim_cached_rows(0, &mut admission)
            .unwrap(),
        2 => assert!(!world
            .registry
            .release_joined_ready(&owner, &mut admission)
            .unwrap()),
        _ => panic!("entry"),
    }
    assert_eq!(admission.charged_work(), own_price);
    {
        let state = world.registry.state.lock().unwrap();
        assert!(state.records[&world.keys[0]].closed_cleanup.queued());
        assert!(state.source_custody[&context.commit(1)].cleanup.queued);
    }
    drop(owner);
    world.drop_owner(1);
    world.drop_owner(2);
    for _ in 0..4 {
        world.admission(1_000_000).unwrap();
    }
    let mut divergences = Vec::new();
    observe::compare(
        &world,
        1,
        1,
        "affording admission after skipped foreign debt",
        &mut divergences,
        true,
    );
    assert!(divergences.is_empty(), "{divergences:?}");
}

fn invalidation_owner() -> SourceInvalidationOwner {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    SourceInvalidationOwner::new(
        WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
        )
        .unwrap(),
        1,
    )
}

#[test]
fn registry_eager_completion_notes_without_sweeping_then_admission_prunes() {
    let context = world::Context::new();
    let mut world = World::new(&context, 1);
    let contacts = super::super::super::post_commit::sweep_contacts();
    world.registry.complete_prepared_source(&context.completed);
    assert_eq!(super::super::super::post_commit::sweep_contacts(), contacts);
    assert!(
        world.registry.state.lock().unwrap().source_custody[&context.commit(1)]
            .cleanup
            .queued
    );
    world.drop_owner(1);
    world.drop_owner(2);
    world.admission(1_000_000).unwrap();
    assert!(!world
        .registry
        .state
        .lock()
        .unwrap()
        .source_custody
        .contains_key(&context.commit(1)));
    let mut divergences = Vec::new();
    observe::compare(
        &world,
        1,
        1,
        "committed completion drained",
        &mut divergences,
        false,
    );
    assert!(divergences.is_empty(), "{divergences:?}");
}
