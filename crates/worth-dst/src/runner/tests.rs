use core::time::Duration;
use std::io;

use worth_execution::time::TimeSampler;

use super::*;
use crate::{ClockFault, Seed};

fn run(seed: u64) -> (Vec<u32>, Trace) {
    let spec = Spec::new(Seed::new(seed), 5, 32)
        .with_clock_fault(1, ClockFault::Advance(Duration::from_secs(2)))
        .with_clock_fault(1, ClockFault::StepWallBack(Duration::from_secs(3)))
        .with_clock_fault(2, ClockFault::Reboot);
    let mut sampler = None;
    let mut schedule = Vec::new();
    let trace = DstRunner::run(&spec, |step, run| {
        let sampler = sampler.get_or_insert_with(|| TimeSampler::new(run.clock()));
        let choice = run.rng().pick(&[2, 4, 7, 11, 19, 21, 23, 42]);
        schedule.push(choice);
        let sample = sampler.sample().unwrap();
        let wall = sample.at().unix_nanos();
        assert_eq!(
            wall,
            if step == 0 {
                5_000_000_000
            } else {
                4_000_000_000
            }
        );
        let source = sample.source().value();
        let seq = sample.seq();
        run.trace().push_json(&format!(
            r#"{{"step":{step},"choice":{choice},"wall":{wall},"source":{source},"seq":{seq}}}"#
        ))
    })
    .unwrap();
    (schedule, trace)
}

#[test]
fn same_seed_reproduces_bytes_and_other_seeds_change_the_schedule() {
    for seed in 0..16 {
        let (schedule, first) = run(seed);
        let (_, repeated) = run(seed);
        assert_eq!(first.as_bytes(), repeated.as_bytes());
        let (different, _) = run(seed + 1);
        assert_ne!(schedule, different);
    }
}

#[test]
fn persistent_naming_seeds_keep_the_original_splitmix_draws() {
    let mut rng = Seed::new(0).rng();
    assert_eq!(
        [rng.next_u64(), rng.next_u64(), rng.next_u64()],
        [
            0xe220_a839_7b1d_cdaf,
            0x6e78_9e6a_a1b9_65f4,
            0x06c4_5d18_8009_454f
        ],
    );
    let mut rng = Seed::new(0).rng();
    assert_eq!([rng.below(17), rng.below(3), rng.below(2)], [12, 0, 1]);
}

#[test]
fn scenario_stops_at_the_first_error() {
    let mut executed = Vec::new();
    let spec = Spec::new(Seed::new(1), 5, 10);
    let result = DstRunner::run(&spec, |step, _| {
        executed.push(step);
        if step == 2 {
            Err(io::ErrorKind::InvalidData)
        } else {
            Ok(())
        }
    });
    assert!(matches!(result, Err(io::ErrorKind::InvalidData)));
    assert_eq!(executed, [0, 1, 2]);
}
