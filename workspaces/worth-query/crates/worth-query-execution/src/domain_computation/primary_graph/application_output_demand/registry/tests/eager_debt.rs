//! An independent eager model observes every step, including admission-free reads.
use super::*;
mod driver;
mod foreign;
mod model;
mod observe;
mod unhold;
mod world;
use model::{Custody, Model, Phase, Row};
use world::World;

#[test]
fn registry_eager_debt_oracle_checks_every_step_for_32_seeds() {
    let context = world::Context::new();
    let mut divergences = Vec::new();
    for seed in 0..32 {
        driver::run(&context, seed, &mut divergences);
        unhold::run(&context, seed, &mut divergences);
    }
    for divergence in &divergences {
        println!("{divergence}");
    }
    assert!(divergences.is_empty(), "eager equivalence is still red");
}

#[test]
fn registry_eager_binding_reads_the_closed_claim_without_an_admission() {
    let context = world::Context::new();
    let mut world = World::new(&context, 1);
    world.drop_owner(1);
    world.drop_owner(2);
    world.bind();
    let state = world.registry.state.lock().unwrap();
    let bound = state.source_custody[&context.commit(0)]
        .bound_sources
        .clone();
    assert_eq!(
        bound,
        Some(vec![super::super::BoundOutputSource {
            scope: context.scope,
            identity: world.keys[0].source.clone(),
        }])
    );
    assert!(state.source_custody[&context.commit(0)]
        .retired_sources
        .is_empty());
}

#[test]
fn registry_eager_claimed_row_transition_itself_leaves_the_note() {
    let context = world::Context::new();
    let world = World::new(&context, 3);
    let mut state = world.registry.state.lock().unwrap();
    assert_eq!(state.records[&world.keys[0]].framework_required_count, 1);
    state.records.get_mut(&world.keys[0]).unwrap().state =
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Superseded,
            "transition",
        ));
    assert!(state.records[&world.keys[0]].closed_cleanup.queued());
}

#[test]
fn registry_eager_owned_closed_note_does_not_name_a_reused_key() {
    let context = world::Context::new();
    let registry = WorthQueryOutputDemandRegistry::default();
    let reused = key("reused-note", 1, 90);
    let old = {
        let mut state = registry.state.lock().unwrap();
        let old = record(
            occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "old",
            )),
            0,
        );
        state.records.insert(reused.clone(), old);
        state.note_closed_row(&reused);
        let old = state.remove_record(&reused).unwrap();
        let mut new = record(occurrence(), DemandState::Admitted, 0);
        new.source_scope = Some(context.scope);
        state.records.insert(reused.clone(), new);
        old
    };
    let owner = registry
        .admit(
            reused.clone(),
            None,
            context.scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .unwrap();
    {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&reused].interests, 1);
        assert_eq!(state.closed_cleanup.len(), 0);
    }
    drop(old); // Even an original row kept outside the map cannot name its replacement.
    drop(owner);
}
