//! Warm installed one-of reads, including the real request and projection handoff.
use crate::native_fragments::{measured, Mode, World};
use std::{hint::black_box, time::Instant};
use worth_query_testkit_decl::fragment_fixture::FormValue;

#[test]
fn warm_installed_fragment_reads_preserve_their_result() {
    let world = World::new(Mode::Measured);
    let expected = FormValue::Measured(measured());
    for _ in 0..16 {
        assert_eq!(world.read().unwrap(), expected);
    }
    let mut samples = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        for _ in 0..128 {
            assert_eq!(black_box(world.read().unwrap()), expected);
        }
        samples.push(started.elapsed().as_micros());
    }
    samples.sort_unstable();
    println!("warm installed fragment reads: 128/sample, 5 samples, microseconds={samples:?}");
}
