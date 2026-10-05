//! The admitted bound a head replay ran past.

/// Effect bytes bound the frames a claim carries. Heap bytes bound what the
/// replay holds at once beyond what its caller already does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadReplayBound {
    EffectBytes,
    HeapBytes,
}

/// A replay stopped at a bound its caller admitted. It says nothing about the
/// media: the same media may pass or fail under a wider bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceededHeadReplayBound {
    pub bound: HeadReplayBound,
    /// What the replay needed. A need that overflows a count is `u64::MAX`.
    pub observed: u64,
    pub admitted: u64,
}

impl ExceededHeadReplayBound {
    /// A need computed with checked arithmetic fits the admitted bytes.
    /// `None` overflowed, and so is past every bound.
    pub(super) fn within(
        bound: HeadReplayBound,
        needed: Option<u64>,
        admitted: u64,
    ) -> Result<(), Self> {
        match needed {
            Some(bytes) if bytes <= admitted => Ok(()),
            past => Err(Self {
                bound,
                observed: past.unwrap_or(u64::MAX),
                admitted,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExceededHeadReplayBound as Exceeded, HeadReplayBound::*};

    #[test]
    fn a_need_past_the_bound_names_the_bound_and_what_it_needed() {
        assert_eq!(Exceeded::within(EffectBytes, Some(64), 64), Ok(()));
        let past = |bound, observed, admitted| Exceeded {
            bound,
            observed,
            admitted,
        };
        assert_eq!(
            Exceeded::within(EffectBytes, Some(65), 64),
            Err(past(EffectBytes, 65, 64))
        );
        assert_eq!(
            Exceeded::within(HeapBytes, Some(1), 0),
            Err(past(HeapBytes, 1, 0))
        );
        assert_eq!(
            Exceeded::within(HeapBytes, None, u64::MAX),
            Err(past(HeapBytes, u64::MAX, u64::MAX))
        );
    }
}
