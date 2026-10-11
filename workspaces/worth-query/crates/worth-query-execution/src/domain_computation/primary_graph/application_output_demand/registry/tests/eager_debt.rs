//! An independent eager model observes every step, including admission-free reads.
use super::*;
mod driver;
mod model;
mod observe;
mod unhold;
mod world;
use model::{Custody, Model, Phase, Row};
use world::World;

// Temporary red oracle: each fix removes observed divergences, never suppresses a step.
const EXPECTED_DIVERGENCES: usize = 32;
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
    assert_eq!(
        divergences.len(),
        EXPECTED_DIVERGENCES,
        "temporary divergence marker; actual eager equivalence requires zero"
    );
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
