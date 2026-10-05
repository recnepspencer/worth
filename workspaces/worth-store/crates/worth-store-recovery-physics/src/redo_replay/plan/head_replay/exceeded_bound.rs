//! The head replay's own budget: the only place that mints its limits. A leaf
//! module, because the authority's declaring module and its descendants can
//! mint.

use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;

worth_proof::authority_marker!(pub HeadReplayBudgetAuthority);

/// Effect bytes bound the frames a claim carries. Heap bytes bound what the
/// replay holds at once beyond what its caller already does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadReplayBound {
    EffectBytes,
    HeapBytes,
}

impl LimitDimension for HeadReplayBound {
    type Authority = HeadReplayBudgetAuthority;
}

/// A replay stopped at a bound its caller admitted. It says nothing about the
/// media: the same media may pass or fail under a wider bound. `observed` is
/// what the replay needed. A need past every count is no limit: the replay
/// names it apart.
pub type ExceededHeadReplayBound = ExhaustedLimit<HeadReplayBound>;

/// One admitted head-replay ceiling, and the one door that refuses past it.
/// Only the replay asks: no other crate can name a ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HeadReplayAllowance {
    bound: HeadReplayBound,
    admitted: u64,
}

impl HeadReplayAllowance {
    pub(super) const fn effect_bytes(admitted: u64) -> Self {
        Self {
            bound: HeadReplayBound::EffectBytes,
            admitted,
        }
    }

    pub(super) const fn heap_bytes(admitted: u64) -> Self {
        Self {
            bound: HeadReplayBound::HeapBytes,
            admitted,
        }
    }

    /// A need within the allowance is returned; past it is this bound's limit.
    pub(super) fn admit(self, needed: u64) -> Result<u64, ExceededHeadReplayBound> {
        if needed <= self.admitted {
            return Ok(needed);
        }
        let counts = LimitCounts::new(needed, self.admitted);
        let refusal =
            Performed::<BudgetRefused, _, _>::record(&HeadReplayBudgetAuthority::witness(), counts);
        Err(ExhaustedLimit::refused(self.bound, refusal))
    }
}

/// What an allowance of `admitted` refuses for `observed`, which must be past
/// it: a test's expected limit, minted through the replay's own door.
#[cfg(any(test, feature = "test-support"))]
pub fn head_replay_limit_for_test(
    bound: HeadReplayBound,
    observed: u64,
    admitted: u64,
) -> ExceededHeadReplayBound {
    assert!(observed > admitted, "a limit is a need past its ceiling");
    let allowance = HeadReplayAllowance { bound, admitted };
    allowance
        .admit(observed)
        .expect_err("a need past the ceiling")
}

#[cfg(test)]
mod tests {
    use super::{HeadReplayAllowance as Allowance, HeadReplayBound::*};

    #[test]
    fn a_need_past_the_bound_names_the_bound_and_what_it_needed() {
        let named = |limit: super::ExceededHeadReplayBound| {
            (limit.dimension(), limit.observed(), limit.admitted())
        };
        assert_eq!(Allowance::effect_bytes(64).admit(64), Ok(64));
        let effect = Allowance::effect_bytes(64).admit(65).unwrap_err();
        assert_eq!(named(effect), (EffectBytes, 65, 64));
        let heap = Allowance::heap_bytes(0).admit(1).unwrap_err();
        assert_eq!(named(heap), (HeapBytes, 1, 0));
        assert_eq!(
            Allowance::heap_bytes(u64::MAX).admit(u64::MAX),
            Ok(u64::MAX)
        );
    }

    #[test]
    #[should_panic(expected = "a limit is a need past its ceiling")]
    fn a_test_limit_within_its_ceiling_is_refused() {
        super::head_replay_limit_for_test(HeapBytes, 4, 4);
    }
}
