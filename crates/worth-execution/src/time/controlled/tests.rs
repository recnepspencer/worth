use core::cmp::Ordering;
use std::sync::Arc;

use worth_foundational::time::{BootEpochMismatch, MonotonicElapsedError};

use super::*;
use crate::time::TimeSampler;

#[test]
fn backward_wall_steps_keep_monotonic_progress_and_reboot_rejects_old_ticks() {
    let clock = Arc::new(ControlledTime::at_unix_secs(2));
    let sampler = TimeSampler::new(clock.clone());
    let start = sampler.monotonic();
    let initial = sampler.sample().unwrap();

    clock.advance(Duration::from_secs(3));
    let advanced = sampler.monotonic();
    assert_eq!(
        advanced.checked_duration_since(start),
        Ok(Duration::from_secs(3))
    );

    clock.step_wall_back(Duration::from_secs(4));
    let stepped = sampler.sample().unwrap();
    assert_eq!(stepped.at().unix_nanos(), 1_000_000_000);
    assert!(stepped.at() < initial.at());
    assert!(stepped.seq() > initial.seq());
    assert_eq!(
        sampler.monotonic().cmp_same_epoch(advanced),
        Ok(Ordering::Equal)
    );

    clock.reboot_epoch();
    let rebooted = sampler.monotonic();
    assert_eq!(rebooted.cmp_same_epoch(advanced), Err(BootEpochMismatch));
    assert_eq!(
        rebooted.checked_duration_since(advanced),
        Err(MonotonicElapsedError::BootEpochMismatch(BootEpochMismatch)),
    );
    assert_eq!(clock.read_monotonic_ticks().0, 0);
    assert_eq!(sampler.wall().unwrap(), stepped.at());
}

#[test]
fn wall_underflow_refuses_until_the_clock_advances_back_into_range() {
    let clock = Arc::new(ControlledTime::at_unix_secs(0));
    let sampler = TimeSampler::new(clock.clone());
    clock.step_wall_back(Duration::from_nanos(2));
    clock.advance(Duration::from_nanos(1));
    assert_eq!(sampler.sample(), Err(TimeUnavailable::WallOutOfRange));
    clock.advance(Duration::from_nanos(1));
    let first = sampler.sample().unwrap();
    assert_eq!(first.at().unix_nanos(), 0);
    assert_eq!(first.seq(), 1);
}

#[test]
fn a_wall_outage_preserves_monotonic_progress_and_issuance_sequence() {
    let clock = Arc::new(ControlledTime::at_unix_nanos(42));
    let sampler = TimeSampler::new(clock.clone());
    let start = sampler.monotonic();
    clock.set_wall_available(false);
    clock.advance(Duration::from_nanos(3));
    assert_eq!(sampler.sample(), Err(TimeUnavailable::WallUnavailable));
    assert_eq!(
        sampler.monotonic().checked_duration_since(start),
        Ok(Duration::from_nanos(3)),
    );
    clock.set_wall_available(true);
    let first = sampler.sample().unwrap();
    assert_eq!(first.at().unix_nanos(), 45);
    assert_eq!(first.seq(), 1);
}

#[test]
fn distinct_clocks_using_one_logical_source_have_incomparable_boots() {
    let source = ClockSourceIdentity::new(12);
    let start = || TimeSampler::new(Arc::new(ControlledTime::with_source_identity(0, source)));
    let one = start();
    let other = start();
    assert_eq!(one.sample(), other.sample());
    assert_eq!(
        one.monotonic().cmp_same_epoch(other.monotonic()),
        Err(BootEpochMismatch)
    );
}
