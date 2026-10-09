//! Source precedence's own budget: the release-custody checks' memory and
//! the page facts' counts. The only place that mints their limits. A leaf
//! module, because the authority's declaring module and its descendants can
//! mint.

use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};

worth_foundational::limit_authority!(pub PhysicsBudgetAuthority);

/// Resident bytes bound what a custody check holds at once. Retained bytes
/// bound what admitted pending-WAL batches keep after their check. Manifest
/// entries and distinct pages and extents bound the page facts of one
/// selected root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsBound {
    ResidentBytes,
    RetainedBytes,
    ManifestEntries,
    DistinctPagesAndExtents,
}

impl LimitDimension for PhysicsBound {
    type Authority = PhysicsBudgetAuthority;
}

/// A custody check stopped at a byte bound its caller admitted. It says
/// nothing about the media: the same media may pass under a wider bound.
/// `observed` is what the check needed. A need past every count is no limit:
/// the check names it apart.
pub type ExceededPhysicsBound = ExhaustedLimit<PhysicsBound>;

/// One admitted custody ceiling, and the one door that refuses past it. Only
/// the custody checks ask: no other crate can name a ceiling, and only this
/// refusal mints a limit, for a need past what it admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PhysicsAllowance {
    bound: PhysicsBound,
    admitted: u64,
}

impl PhysicsAllowance {
    pub(super) const fn resident_bytes(admitted: u64) -> Self {
        Self {
            bound: PhysicsBound::ResidentBytes,
            admitted,
        }
    }

    pub(super) const fn retained_bytes(admitted: u64) -> Self {
        Self {
            bound: PhysicsBound::RetainedBytes,
            admitted,
        }
    }

    pub(super) const fn manifest_entries(admitted: u64) -> Self {
        Self {
            bound: PhysicsBound::ManifestEntries,
            admitted,
        }
    }

    pub(super) const fn distinct_pages_and_extents(admitted: u64) -> Self {
        Self {
            bound: PhysicsBound::DistinctPagesAndExtents,
            admitted,
        }
    }

    /// A need within the allowance passes; past it is this bound's limit.
    pub(super) fn admit(self, needed: u64) -> Result<(), ExceededPhysicsBound> {
        if needed <= self.admitted {
            return Ok(());
        }
        let counts = LimitCounts::new(needed, self.admitted);
        Err(PhysicsBudgetAuthority::refuse(self.bound, counts))
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through the checks' own door.
#[cfg(any(test, feature = "test-support"))]
pub fn physics_limit_for_test(
    bound: PhysicsBound,
    observed: u64,
    admitted: u64,
) -> ExceededPhysicsBound {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    let allowance = PhysicsAllowance { bound, admitted };
    allowance
        .admit(observed)
        .expect_err("a need past the ceiling")
}

#[cfg(test)]
mod tests {
    use super::{PhysicsAllowance as Allowance, PhysicsBound::*};

    #[test]
    fn bytes_past_the_allowance_name_the_bound_and_both_counts() {
        let named = |limit: super::ExceededPhysicsBound| {
            (limit.dimension(), limit.observed(), limit.admitted())
        };
        assert_eq!(Allowance::resident_bytes(64).admit(64), Ok(()));
        let resident = Allowance::resident_bytes(64).admit(65).unwrap_err();
        assert_eq!(named(resident), (ResidentBytes, 65, 64));
        assert_eq!(Allowance::retained_bytes(8).admit(8), Ok(()));
        let retained = Allowance::retained_bytes(8).admit(9).unwrap_err();
        assert_eq!(named(retained), (RetainedBytes, 9, 8));
        assert_eq!(Allowance::resident_bytes(u64::MAX).admit(u64::MAX), Ok(()));
        let entries = Allowance::manifest_entries(3).admit(4).unwrap_err();
        assert_eq!(named(entries), (ManifestEntries, 4, 3));
        let distinct = Allowance::distinct_pages_and_extents(5)
            .admit(7)
            .unwrap_err();
        assert_eq!(named(distinct), (DistinctPagesAndExtents, 7, 5));
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::physics_limit_for_test(RetainedBytes, 8, 8);
    }
}
