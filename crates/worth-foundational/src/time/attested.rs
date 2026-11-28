use super::{ClockSourceIdentity, TimeSourceEpoch};

/// Unix wall time with its source epoch, persisted only as recorded evidence.
/// Ordering is by nanoseconds and then source epoch. Wall time may step
/// backward, even within one source epoch.
///
/// Constructing this portable value carries no authority to issue a sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttestedTime {
    unix_nanos: u64,
    source: TimeSourceEpoch,
}

impl AttestedTime {
    pub const fn new(unix_nanos: u64, source: TimeSourceEpoch) -> Self {
        Self { unix_nanos, source }
    }

    pub const fn unix_nanos(self) -> u64 {
        self.unix_nanos
    }

    pub const fn source(self) -> TimeSourceEpoch {
        self.source
    }
}

/// A wall-clock observation could not be issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnavailable {
    /// The adapter had no wall reading.
    WallUnavailable,
    /// The wall reading was outside the representable Unix nanosecond range.
    WallOutOfRange,
    /// The source identity no longer matched the sampler's binding.
    SourceMismatch {
        expected: ClockSourceIdentity,
        observed: ClockSourceIdentity,
    },
    /// This sampler has used every available sequence number.
    SequenceExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_evidence_has_a_total_coordinate_then_source_order() {
        let source = |id| TimeSourceEpoch::new(ClockSourceIdentity::new(id));
        for nanos in [0, 1, u64::MAX - 1] {
            let earlier = AttestedTime::new(nanos, source(2));
            assert!(earlier < AttestedTime::new(nanos + 1, source(1)));
            assert!(AttestedTime::new(nanos, source(1)) < earlier);
        }
    }
}
