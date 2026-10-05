//! The release-custody head walk's own budget: the only place that mints its
//! limits. A leaf module, because the authority's declaring module and its
//! descendants can mint.

use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;

worth_proof::authority_marker!(pub ReleaseCustodyHeadWalkBudgetAuthority);

/// What a release-custody head walk ran out of. Nodes bound the blocks the
/// walk admits; resident bytes bound what it holds at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCustodyHeadWalkBound {
    Nodes,
    ResidentBytes,
}

impl LimitDimension for ReleaseCustodyHeadWalkBound {
    type Authority = ReleaseCustodyHeadWalkBudgetAuthority;
}

/// A walk stopped at a bound its caller admitted. It says nothing about the
/// tree beyond the count that passed the bound.
pub type ExceededReleaseCustodyHeadWalkBound = ExhaustedLimit<ReleaseCustodyHeadWalkBound>;

/// One admitted ceiling of the walk, and the one door that refuses past it.
/// Only the walk asks: no other crate can name a ceiling. Its refusal mints
/// a limit only for a count past what it admitted.
#[derive(Debug, Clone, Copy)]
pub(super) struct ReleaseCustodyHeadWalkAllowance {
    bound: ReleaseCustodyHeadWalkBound,
    admitted: u64,
}

impl ReleaseCustodyHeadWalkAllowance {
    pub(super) const fn nodes(admitted: u64) -> Self {
        Self {
            bound: ReleaseCustodyHeadWalkBound::Nodes,
            admitted,
        }
    }

    pub(super) const fn resident_bytes(admitted: u64) -> Self {
        Self {
            bound: ReleaseCustodyHeadWalkBound::ResidentBytes,
            admitted,
        }
    }

    /// `observed` within the allowance passes; past it is this bound's limit.
    pub(super) fn admit(self, observed: u64) -> Result<(), ExceededReleaseCustodyHeadWalkBound> {
        if observed <= self.admitted {
            return Ok(());
        }
        let counts = LimitCounts::new(observed, self.admitted);
        let refusal = Performed::<BudgetRefused, _, _>::record(
            &ReleaseCustodyHeadWalkBudgetAuthority::witness(),
            counts,
        );
        Err(ExhaustedLimit::refused(self.bound, refusal))
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through the walk's own door.
#[cfg(any(test, feature = "test-support"))]
pub fn release_custody_head_walk_limit_for_test(
    bound: ReleaseCustodyHeadWalkBound,
    observed: u64,
    admitted: u64,
) -> ExceededReleaseCustodyHeadWalkBound {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    let allowance = ReleaseCustodyHeadWalkAllowance { bound, admitted };
    allowance
        .admit(observed)
        .expect_err("a need past the ceiling")
}

#[cfg(test)]
mod tests {
    use super::{ReleaseCustodyHeadWalkAllowance, ReleaseCustodyHeadWalkBound as Bound};

    #[test]
    fn a_count_past_the_allowance_is_that_bound_with_both_counts() {
        assert_eq!(ReleaseCustodyHeadWalkAllowance::nodes(3).admit(3), Ok(()));
        let nodes = ReleaseCustodyHeadWalkAllowance::nodes(3)
            .admit(4)
            .unwrap_err();
        assert_eq!(
            (nodes.dimension(), nodes.observed(), nodes.admitted()),
            (Bound::Nodes, 4, 3)
        );
        assert_eq!(
            ReleaseCustodyHeadWalkAllowance::resident_bytes(64).admit(64),
            Ok(())
        );
        let resident = ReleaseCustodyHeadWalkAllowance::resident_bytes(64)
            .admit(65)
            .unwrap_err();
        assert_eq!(
            (
                resident.dimension(),
                resident.observed(),
                resident.admitted()
            ),
            (Bound::ResidentBytes, 65, 64)
        );
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::release_custody_head_walk_limit_for_test(Bound::Nodes, 3, 3);
    }
}
