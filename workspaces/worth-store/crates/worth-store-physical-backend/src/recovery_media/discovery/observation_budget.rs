//! A filesystem observation's own budget: the only place that mints its
//! limits. A leaf module, because the authority's declaring module and its
//! descendants can mint.

use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};

worth_foundational::limit_authority!(pub FilesystemObservationBudgetAuthority);

/// What a filesystem observation ran out of:
/// - reads: the artifacts one observation may read;
/// - entries: the names one WAL listing may hold, or the WAL observations one
///   observation may issue;
/// - observation bytes: the bytes one observation may read in all;
/// - requested bytes: the ceiling a caller named for one read or one WAL
///   inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemObservationBound {
    Reads,
    Entries,
    ObservationBytes,
    RequestedBytes,
}

impl LimitDimension for FilesystemObservationBound {
    type Authority = FilesystemObservationBudgetAuthority;
}

/// An observation stopped at a bound it was admitted. It says nothing about
/// the media: the same media may pass under a wider bound. `observed` is what
/// the observation needed. A need past every count is no limit: the
/// observation names that count apart.
pub type ExceededFilesystemObservationBound = ExhaustedLimit<FilesystemObservationBound>;

/// One admitted observation ceiling, and the doors that refuse past it.
/// Only the observation asks: no other crate can name a ceiling. Its refusal
/// mints a limit only for a need past what it admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FilesystemObservationAllowance {
    bound: FilesystemObservationBound,
    admitted: u64,
}

impl FilesystemObservationAllowance {
    pub(super) const fn reads(admitted: u64) -> Self {
        Self::of(FilesystemObservationBound::Reads, admitted)
    }

    pub(super) const fn entries(admitted: u64) -> Self {
        Self::of(FilesystemObservationBound::Entries, admitted)
    }

    pub(super) const fn observation_bytes(admitted: u64) -> Self {
        Self::of(FilesystemObservationBound::ObservationBytes, admitted)
    }

    pub(super) const fn requested_bytes(admitted: u64) -> Self {
        Self::of(FilesystemObservationBound::RequestedBytes, admitted)
    }

    const fn of(bound: FilesystemObservationBound, admitted: u64) -> Self {
        Self { bound, admitted }
    }

    /// A need within the allowance is returned; past it is this bound's limit.
    pub(super) fn admit(self, needed: u64) -> Result<u64, ExceededFilesystemObservationBound> {
        if needed <= self.admitted {
            Ok(needed)
        } else {
            Err(self.refuse(needed))
        }
    }

    /// A limit a narrower allowance of the same bound refused, read where
    /// `held` more of it was already spent beside it: both counts move by
    /// what is held, so the distance between them stays the one the narrower
    /// allowance found. A moved count past every count is `None`: no limit
    /// can state it.
    pub(super) fn held_beside(
        narrower: ExceededFilesystemObservationBound,
        held: u64,
    ) -> Option<ExceededFilesystemObservationBound> {
        let admitted = narrower.admitted().checked_add(held)?;
        let observed = narrower.observed().checked_add(held)?;
        Some(Self::of(narrower.dimension(), admitted).refuse(observed))
    }

    fn refuse(self, observed: u64) -> ExceededFilesystemObservationBound {
        let counts = LimitCounts::new(observed, self.admitted);
        FilesystemObservationBudgetAuthority::refuse(self.bound, counts)
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through the observation's own door.
#[cfg(any(test, feature = "test-support"))]
pub fn filesystem_observation_limit_for_test(
    bound: FilesystemObservationBound,
    observed: u64,
    admitted: u64,
) -> ExceededFilesystemObservationBound {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    FilesystemObservationAllowance::of(bound, admitted)
        .admit(observed)
        .expect_err("a need past the ceiling")
}

#[cfg(test)]
mod tests {
    use super::{FilesystemObservationAllowance as Allowance, FilesystemObservationBound::*};

    fn named(
        limit: super::ExceededFilesystemObservationBound,
    ) -> (super::FilesystemObservationBound, u64, u64) {
        (limit.dimension(), limit.observed(), limit.admitted())
    }

    #[test]
    fn each_allowance_refuses_past_itself_with_its_bound_and_both_counts() {
        for (allowance, bound) in [
            (Allowance::reads(3), Reads),
            (Allowance::entries(3), Entries),
            (Allowance::observation_bytes(3), ObservationBytes),
            (Allowance::requested_bytes(3), RequestedBytes),
        ] {
            assert_eq!(allowance.admit(3), Ok(3));
            assert_eq!(named(allowance.admit(4).unwrap_err()), (bound, 4, 3));
        }
        assert_eq!(Allowance::reads(u64::MAX).admit(u64::MAX), Ok(u64::MAX));
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::filesystem_observation_limit_for_test(Reads, 3, 3);
    }

    #[test]
    fn held_bytes_move_both_counts_and_keep_the_bound() {
        let inner = Allowance::requested_bytes(10).admit(12).unwrap_err();
        assert_eq!(
            Allowance::held_beside(inner, 22).map(named),
            Some((RequestedBytes, 34, 32))
        );
        // A moved count past every count is no limit.
        assert_eq!(Allowance::held_beside(inner, u64::MAX - 11), None);
    }
}
