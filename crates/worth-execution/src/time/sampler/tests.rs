use super::*;
use crate::time::ControlledTime;

#[test]
fn repeated_wall_readings_issue_unique_contiguous_sequences_across_callers() {
    let sampler = TimeSampler::new(Arc::new(ControlledTime::at_unix_secs(42)));
    let at = sampler.wall().unwrap();
    const SAMPLES_PER_CALLER: u64 = 32;

    let mut issued = std::thread::scope(|scope| {
        let sample = || {
            let issued = (0..SAMPLES_PER_CALLER)
                .map(|_| {
                    let sample = sampler.sample().unwrap();
                    assert_eq!(sample.at(), at);
                    sample.seq()
                })
                .collect::<Vec<_>>();
            assert!(issued.windows(2).all(|pair| pair[0] < pair[1]));
            issued
        };
        let other = scope.spawn(sample);
        let mut issued = sample();
        issued.extend(other.join().unwrap());
        issued
    });

    issued.sort_unstable();
    assert!(issued.into_iter().eq(1..=2 * SAMPLES_PER_CALLER));
}

struct DriftingSource {
    identity: AtomicU64,
    change_on_read: std::sync::atomic::AtomicBool,
}

impl RawClockAdapter for DriftingSource {
    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        if self.change_on_read.load(Ordering::Relaxed) {
            self.identity.store(2, Ordering::Relaxed);
        }
        Ok(42)
    }

    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        (0, BootEpoch::new(0))
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        ClockSourceIdentity::new(self.identity.load(Ordering::Relaxed))
    }
}

#[test]
fn source_drift_before_or_during_a_read_refuses_without_issuing() {
    let adapter = Arc::new(DriftingSource {
        identity: AtomicU64::new(1),
        change_on_read: true.into(),
    });
    let sampler = TimeSampler::new(adapter.clone());
    let expected = TimeUnavailable::SourceMismatch {
        expected: ClockSourceIdentity::new(1),
        observed: ClockSourceIdentity::new(2),
    };
    assert_eq!(sampler.sample(), Err(expected));
    adapter.change_on_read.store(false, Ordering::Relaxed);
    assert_eq!(sampler.sample(), Err(expected));
    adapter.identity.store(1, Ordering::Relaxed);
    assert_eq!(sampler.sample().unwrap().seq(), 1);
}

#[test]
fn sequence_exhaustion_never_reissues_an_old_sample() {
    let sampler = TimeSampler::new(Arc::new(ControlledTime::at_unix_secs(0)));
    sampler.seq.store(u64::MAX - 1, Ordering::Relaxed);
    assert_eq!(sampler.sample().unwrap().seq(), u64::MAX);
    assert_eq!(sampler.sample(), Err(TimeUnavailable::SequenceExhausted));
    assert_eq!(sampler.sample(), Err(TimeUnavailable::SequenceExhausted));
}
