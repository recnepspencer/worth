//! The admitted bound a root-history check ran past.

/// Entries bound how many routes, segments, free entries or records one
/// inventory view or one step may hold. Scratch bytes bound what a check may
/// hold at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootHistoryBound {
    Entries,
    ScratchBytes,
}

/// A check stopped at a bound its caller admitted. It says nothing about the
/// media: the same media may pass or fail under a wider bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceededRootHistoryBound {
    pub bound: RootHistoryBound,
    /// At least what the check needed. A need that overflows a count is
    /// `u64::MAX`. Where the host refused scratch the bound admitted, this is
    /// the scratch asked for, and is not past `admitted`.
    pub observed: u64,
    pub admitted: u64,
}

impl ExceededRootHistoryBound {
    pub(crate) const fn entries(observed: u64, admitted: u64) -> Self {
        Self {
            bound: RootHistoryBound::Entries,
            observed,
            admitted,
        }
    }

    pub(crate) const fn scratch(observed: u64, admitted: u64) -> Self {
        Self {
            bound: RootHistoryBound::ScratchBytes,
            observed,
            admitted,
        }
    }

    /// Every count fits an admitted, nonzero entry bound.
    pub(crate) fn entries_within(
        counts: impl IntoIterator<Item = usize>,
        admitted: u64,
    ) -> Result<(), Self> {
        let past = counts
            .into_iter()
            .map(|count| count as u64)
            .chain((admitted == 0).then_some(1))
            .find(|count| *count > admitted);
        past.map_or(Ok(()), |count| Err(Self::entries(count, admitted)))
    }

    /// Scratch computed with checked arithmetic fits the admitted bytes.
    /// `None` overflowed, and so is past every bound.
    pub(crate) fn scratch_within(needed: Option<u64>, admitted: u64) -> Result<u64, Self> {
        match needed {
            Some(bytes) if bytes <= admitted => Ok(bytes),
            past => Err(Self::scratch(past.unwrap_or(u64::MAX), admitted)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExceededRootHistoryBound as Exceeded, RootHistoryBound};

    #[test]
    fn the_first_count_past_the_entries_is_the_one_named() {
        assert_eq!(Exceeded::entries_within([3, 4, 4], 4), Ok(()));
        assert_eq!(
            Exceeded::entries_within([3, 5, 9], 4),
            Err(Exceeded::entries(5, 4))
        );
        // No view fits a bound of nothing, not even an empty one.
        assert_eq!(
            Exceeded::entries_within([0, 0], 0),
            Err(Exceeded::entries(1, 0))
        );
        assert_eq!(Exceeded::entries(5, 4).bound, RootHistoryBound::Entries);
    }

    #[test]
    fn scratch_past_the_bytes_names_what_it_needed() {
        assert_eq!(Exceeded::scratch_within(Some(64), 64), Ok(64));
        assert_eq!(
            Exceeded::scratch_within(Some(65), 64),
            Err(Exceeded::scratch(65, 64))
        );
        assert_eq!(
            Exceeded::scratch_within(None, u64::MAX),
            Err(Exceeded::scratch(u64::MAX, u64::MAX))
        );
        assert_eq!(
            Exceeded::scratch(65, 64).bound,
            RootHistoryBound::ScratchBytes
        );
    }
}
