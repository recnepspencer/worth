//! The root-history checks' own budget: the only place that mints their
//! limits. A leaf module, because the authority's declaring module and its
//! descendants can mint.

use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};

worth_foundational::limit_authority!(pub RootHistoryBudgetAuthority);

/// Entries bound how many routes, segments, free entries or records one
/// inventory view or one step may hold. Scratch bytes bound what a check may
/// hold at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootHistoryBound {
    Entries,
    ScratchBytes,
}

impl LimitDimension for RootHistoryBound {
    type Authority = RootHistoryBudgetAuthority;
}

/// A check stopped at a bound its caller admitted. It says nothing about the
/// media: the same media may pass or fail under a wider bound.
///
/// `observed` is what the check needed. A need past every count is no limit:
/// the check names it apart. Where the host refused scratch the bound
/// admitted, `observed` is the scratch asked for, and is not past `admitted`.
pub type ExceededRootHistoryBound = ExhaustedLimit<RootHistoryBound>;

/// One admitted root-history ceiling, and the doors that refuse past it.
/// Only the root-history checks ask: no other crate can name a ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RootHistoryAllowance {
    bound: RootHistoryBound,
    admitted: u64,
}

impl RootHistoryAllowance {
    pub(super) const fn entries(admitted: u64) -> Self {
        Self {
            bound: RootHistoryBound::Entries,
            admitted,
        }
    }

    pub(super) const fn scratch_bytes(admitted: u64) -> Self {
        Self {
            bound: RootHistoryBound::ScratchBytes,
            admitted,
        }
    }

    /// A need within the allowance is returned; past it is this bound's limit.
    pub(super) fn admit(self, needed: u64) -> Result<u64, ExceededRootHistoryBound> {
        if needed <= self.admitted {
            Ok(needed)
        } else {
            Err(self.refuse(needed))
        }
    }

    /// Every count fits the allowance, which no view fits when it is zero,
    /// not even an empty one. The first count past it is the one named.
    pub(super) fn admit_each(
        self,
        counts: impl IntoIterator<Item = usize>,
    ) -> Result<(), ExceededRootHistoryBound> {
        counts
            .into_iter()
            .map(|count| count as u64)
            .chain((self.admitted == 0).then_some(1))
            .try_for_each(|count| self.admit(count).map(drop))
    }

    /// The host refused `asked` bytes this allowance admitted: the same
    /// refusal, though `asked` is not past it.
    pub(super) fn host_refused(self, asked: u64) -> ExceededRootHistoryBound {
        self.refuse(asked)
    }

    /// A limit a narrower allowance refused, read where `held` more bytes sit
    /// beside it: both counts move by what is held, so the distance between
    /// them stays the one the narrower allowance found. Entries hold no
    /// bytes and stay as refused. A moved count past every count is `None`:
    /// no limit can state it.
    pub(super) fn held_beside(
        narrower: ExceededRootHistoryBound,
        held: u64,
    ) -> Option<ExceededRootHistoryBound> {
        match narrower.dimension() {
            RootHistoryBound::Entries => Some(narrower),
            RootHistoryBound::ScratchBytes => {
                let admitted = narrower.admitted().checked_add(held)?;
                let observed = narrower.observed().checked_add(held)?;
                Some(Self::scratch_bytes(admitted).refuse(observed))
            }
        }
    }

    fn refuse(self, observed: u64) -> ExceededRootHistoryBound {
        let counts = LimitCounts::new(observed, self.admitted);
        RootHistoryBudgetAuthority::refuse(self.bound, counts)
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through the checks' own door.
#[cfg(any(test, feature = "test-support"))]
pub fn root_history_limit_for_test(
    bound: RootHistoryBound,
    observed: u64,
    admitted: u64,
) -> ExceededRootHistoryBound {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    let allowance = RootHistoryAllowance { bound, admitted };
    allowance
        .admit(observed)
        .expect_err("a need past the ceiling")
}

/// What an entries allowance of `admitted` refuses for `observed`.
#[cfg(test)]
pub(crate) fn entries_past(observed: u64, admitted: u64) -> ExceededRootHistoryBound {
    root_history_limit_for_test(RootHistoryBound::Entries, observed, admitted)
}

/// What a scratch allowance of `admitted` refuses for `observed`.
#[cfg(test)]
pub(crate) fn scratch_past(observed: u64, admitted: u64) -> ExceededRootHistoryBound {
    root_history_limit_for_test(RootHistoryBound::ScratchBytes, observed, admitted)
}

#[cfg(test)]
mod tests {
    use super::{RootHistoryAllowance as Allowance, RootHistoryBound};

    fn named(limit: super::ExceededRootHistoryBound) -> (RootHistoryBound, u64, u64) {
        (limit.dimension(), limit.observed(), limit.admitted())
    }

    #[test]
    fn the_first_count_past_the_entries_is_the_one_named() {
        assert_eq!(Allowance::entries(4).admit_each([3, 4, 4]), Ok(()));
        let past = Allowance::entries(4).admit_each([3, 5, 9]).unwrap_err();
        assert_eq!(named(past), (RootHistoryBound::Entries, 5, 4));
        // No view fits a bound of nothing, not even an empty one.
        let nothing = Allowance::entries(0).admit_each([0, 0]).unwrap_err();
        assert_eq!(named(nothing), (RootHistoryBound::Entries, 1, 0));
    }

    #[test]
    fn scratch_past_the_bytes_names_what_it_needed() {
        assert_eq!(Allowance::scratch_bytes(64).admit(64), Ok(64));
        let past = Allowance::scratch_bytes(64).admit(65).unwrap_err();
        assert_eq!(named(past), (RootHistoryBound::ScratchBytes, 65, 64));
        assert_eq!(
            Allowance::scratch_bytes(u64::MAX).admit(u64::MAX),
            Ok(u64::MAX)
        );
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::root_history_limit_for_test(RootHistoryBound::Entries, 3, 3);
    }

    #[test]
    fn a_host_refusal_names_what_was_asked_within_the_bound() {
        let refused = Allowance::scratch_bytes(64).host_refused(48);
        assert_eq!(named(refused), (RootHistoryBound::ScratchBytes, 48, 64));
    }

    #[test]
    fn held_bytes_move_both_scratch_counts_and_leave_entries_alone() {
        let scratch = Allowance::scratch_bytes(40).admit(50).unwrap_err();
        assert_eq!(
            Allowance::held_beside(scratch, 24).map(named),
            Some((RootHistoryBound::ScratchBytes, 74, 64))
        );
        let entries = Allowance::entries(2).admit(3).unwrap_err();
        assert_eq!(Allowance::held_beside(entries, 24), Some(entries));
        // A moved count past every count is no limit.
        assert_eq!(Allowance::held_beside(scratch, u64::MAX - 45), None);
    }
}
