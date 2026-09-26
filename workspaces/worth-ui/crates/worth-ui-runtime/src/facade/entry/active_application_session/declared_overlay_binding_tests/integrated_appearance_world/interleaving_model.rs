//! A deterministic interleaving model: seeded runs mix Motion ticks prepared
//! and held, commits, pre-effect rejections of ticks and publications, wheel
//! notches and retargets, direct owner edits, content resizes, publications
//! held open by the host, and surface rebinds. Motion ticks and notches go
//! through the entries the native shell runs; publications go through the
//! session's own publication path. After every step the displayed Motion
//! sample, the Scroll offset, both hit-test lanes and the cursor must equal
//! the geometry the latest witness proved, and the host's drawn text is the
//! oracle for what is on screen. Each run then comes to rest: the attempt
//! the host holds completes, the held tick presents, and every settle Scroll
//! holds lands where the pose rests.
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

/// The first eight seeds, one that reaches a settle arriving behind a
/// resize, and the seeds that once found a sample the host kept drawing
/// where hit testing no longer read it, a sample a frame in flight displaced
/// once it landed, and a retarget that carried the content past rest.
const SEEDS: [u64; 21] = [
    0x5eed_0001,
    0x5eed_0002,
    0x5eed_0003,
    0x5eed_0004,
    0x5eed_0005,
    0x5eed_0006,
    0x5eed_0007,
    0x5eed_0008,
    0x5eed_0028,
    0x5eed_001e,
    0x5eed_003c,
    0x5eed_005e,
    0x5eed_0075,
    0x5eed_009e,
    0x5eed_00e2,
    0x5eed_004b,
    0x5eed_006a,
    0x5eed_00e8,
    0x5eed_0182,
    0x5eed_01e3,
    0x5eed_01f5,
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
        let label = format!("seed {seed:#x} at rest");
        model.rest(&label);
        reached.extend(witness::assert_witnessed(&model, &label));
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
