//! A deterministic interleaving model: seeded runs mix Motion ticks prepared
//! and held, commits, pre-effect rejections of ticks and publications, wheel
//! notches and retargets, direct owner edits, content resizes, publications
//! held open by the host, and surface rebinds. Motion ticks and notches go
//! through the entries the native shell runs; publications go through the
//! session's own publication path. After every step the displayed Motion
//! sample, the Scroll offset, both hit-test lanes and the cursor must equal
//! the geometry the latest witness proved, and the host's drawn text is the
//! oracle for what is on screen.
//!
//! The seeds are fixed, so a failure names the seed and step that reproduce
//! it. Every kind of operation must take effect somewhere across the runs, so
//! the model cannot pass by refusing everything it tries, and every state the
//! witness tells apart must be reached.

#[path = "interleaving_model/steps.rs"]
mod steps;
#[path = "interleaving_model/witness.rs"]
mod witness;

use std::collections::BTreeSet;
use steps::{Model, Step};

const SEEDS: [u64; 8] = [
    0x5eed_0001,
    0x5eed_0002,
    0x5eed_0003,
    0x5eed_0004,
    0x5eed_0005,
    0x5eed_0006,
    0x5eed_0007,
    0x5eed_0008,
];
const STEPS: usize = 64;

/// A 64-bit linear congruential generator: the model's only source of
/// choice, so every run is reproducible from its seed.
fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state >> 33
}

#[test]
fn every_interleaving_keeps_displayed_geometry_on_the_latest_witness() {
    let mut effects = BTreeSet::new();
    let mut reached = BTreeSet::new();
    for seed in SEEDS {
        let mut state = seed;
        let mut model = Model::launch();
        reached.extend(witness::assert_witnessed(
            &model,
            &format!("seed {seed:#x} launch"),
        ));
        for index in 0..STEPS {
            let step = Step::CHOICES[next(&mut state) as usize % Step::CHOICES.len()];
            let effect = model.apply(step, next(&mut state));
            effects.extend(effect);
            reached.extend(witness::assert_witnessed(
                &model,
                &format!("seed {seed:#x} step {index} {step:?} took {effect:?}"),
            ));
        }
        model.finish();
    }
    let missing = Step::EVERY
        .into_iter()
        .filter(|step| !effects.contains(step))
        .collect::<Vec<_>>();
    assert!(missing.is_empty(), "no run took effect for {missing:?}");
    let unreached = witness::Reached::EVERY
        .into_iter()
        .filter(|state| !reached.contains(state))
        .collect::<Vec<_>>();
    assert!(unreached.is_empty(), "no run reached {unreached:?}");
}
