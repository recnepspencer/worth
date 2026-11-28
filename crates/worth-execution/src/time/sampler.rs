use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use worth_foundational::time::{
    AttestedTime, BootEpoch, ClockSourceIdentity, MonotonicInstant, TimeSourceEpoch,
    TimeUnavailable,
};

/// A composition-supplied source of raw time readings. Monotonic ticks are
/// nanoseconds on the adapter's current boot timeline. Its wall source identity
/// must change when the source binding changes.
pub trait RawClockAdapter: Send + Sync + 'static {
    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable>;
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch);
    fn source_identity(&self) -> ClockSourceIdentity;
}

/// Wall evidence issued by a [`TimeSampler`]. The sequence identifies issuance
/// order within that sampler, including repeated or backward wall readings.
///
/// ```compile_fail,E0451
/// use worth_execution::time::RecordedTimeSample;
/// use worth_foundational::time::{AttestedTime, ClockSourceIdentity, TimeSourceEpoch};
///
/// let source = ClockSourceIdentity::new(1);
/// let at = AttestedTime::new(42, TimeSourceEpoch::new(source));
/// let _forged = RecordedTimeSample { at, seq: 1, source };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordedTimeSample {
    at: AttestedTime,
    seq: u64,
    source: ClockSourceIdentity,
}

impl RecordedTimeSample {
    pub const fn at(self) -> AttestedTime {
        self.at
    }

    pub const fn seq(self) -> u64 {
        self.seq
    }

    pub const fn source(self) -> ClockSourceIdentity {
        self.source
    }
}

/// Binds one raw source and issues a fresh sequence for each accepted sample.
///
/// ```
/// use std::sync::Arc;
/// use worth_execution::time::{ControlledTime, TimeSampler};
///
/// let sampler = TimeSampler::new(Arc::new(ControlledTime::at_unix_secs(42)));
/// let sample = sampler.sample().unwrap();
/// assert_eq!(sample.at().unix_nanos(), 42_000_000_000);
/// assert_eq!(sample.at().source().identity(), sample.source());
/// assert_eq!(sample.seq(), 1);
/// ```
pub struct TimeSampler {
    adapter: Arc<dyn RawClockAdapter>,
    seq: AtomicU64,
    source: ClockSourceIdentity,
}

impl TimeSampler {
    pub fn new(adapter: Arc<dyn RawClockAdapter>) -> Self {
        let source = adapter.source_identity();
        Self {
            adapter,
            seq: AtomicU64::new(0),
            source,
        }
    }

    /// Reads wall evidence and validates the bound identity around the read.
    pub fn wall(&self) -> Result<AttestedTime, TimeUnavailable> {
        self.check_source()?;
        let unix_nanos = self.adapter.read_wall_unix_nanos()?;
        self.check_source()?;
        Ok(AttestedTime::new(
            unix_nanos,
            TimeSourceEpoch::new(self.source),
        ))
    }

    pub fn monotonic(&self) -> MonotonicInstant {
        let (ticks, boot) = self.adapter.read_monotonic_ticks();
        MonotonicInstant::new(ticks, boot)
    }

    /// Issues after validation. Failed reads and failed source checks leave the
    /// sequence untouched. Concurrent calls are ordered by sequence allocation.
    #[allow(
        deprecated,
        reason = "fetch_update supports the workspace Rust 1.94 minimum"
    )]
    pub fn sample(&self) -> Result<RecordedTimeSample, TimeUnavailable> {
        let at = self.wall()?;
        let previous = self
            .seq
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |seq| {
                seq.checked_add(1)
            })
            .map_err(|_| TimeUnavailable::SequenceExhausted)?;
        Ok(RecordedTimeSample {
            at,
            seq: previous + 1,
            source: self.source,
        })
    }

    fn check_source(&self) -> Result<(), TimeUnavailable> {
        let observed = self.adapter.source_identity();
        if observed == self.source {
            Ok(())
        } else {
            Err(TimeUnavailable::SourceMismatch {
                expected: self.source,
                observed,
            })
        }
    }
}

#[cfg(test)]
mod tests;
