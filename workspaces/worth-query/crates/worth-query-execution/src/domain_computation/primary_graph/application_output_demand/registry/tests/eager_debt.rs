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
const EXPECTED_DIVERGENCES: usize = 168;
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
