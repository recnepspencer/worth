use core::fmt::Debug;

use worth_proof::{ActionMarker, AuthorityMarker, Performed};

/// What ran out. The owner of a budget implements this on its own enum.
///
/// `Authority` ties the dimension to that owner: only a `Performed` recorded
/// under the owner's sealed witness can mint an [`ExhaustedLimit`] of it.
/// Coherence admits one implementation per dimension, and the orphan rule
/// keeps it in the crate that declares the enum.
pub trait LimitDimension: Copy + Eq + Debug {
    type Authority: AuthorityMarker;
}

/// The counts a budget held when it refused.
///
/// `observed` is at least what the work needed (or, where the host refused
/// memory the budget admitted, what was asked for). `admitted` is what the
/// budget allowed. Neither is ever invented where the refusal is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitCounts {
    observed: u64,
    admitted: u64,
}

impl LimitCounts {
    pub const fn new(observed: u64, admitted: u64) -> Self {
        Self { observed, admitted }
    }

    pub const fn observed(self) -> u64 {
        self.observed
    }

    pub const fn admitted(self) -> u64 {
        self.admitted
    }
}

/// The action a budget records when it refuses: the evidence
/// [`ExhaustedLimit::refused`] consumes.
#[derive(Debug)]
pub enum BudgetRefused {}

impl ActionMarker for BudgetRefused {}

/// A limit that ran out, carrying the dimension and the owning budget's counts.
///
/// Fields are private and the one constructor consumes the owner's refusal
/// evidence, so damage cannot be reported as a limit and a limit cannot be
/// built from numbers no budget produced. The limit itself may be copied; the
/// `Performed` it was minted from may not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExhaustedLimit<D: LimitDimension> {
    dimension: D,
    counts: LimitCounts,
}

impl<D: LimitDimension> ExhaustedLimit<D> {
    /// Mint the limit from the owning budget's recorded refusal.
    pub fn refused(
        dimension: D,
        refusal: Performed<BudgetRefused, D::Authority, LimitCounts>,
    ) -> Self {
        Self {
            dimension,
            counts: refusal.into_outcome(),
        }
    }

    pub const fn dimension(&self) -> D {
        self.dimension
    }

    pub const fn counts(&self) -> LimitCounts {
        self.counts
    }

    pub const fn observed(&self) -> u64 {
        self.counts.observed
    }

    pub const fn admitted(&self) -> u64 {
        self.counts.admitted
    }
}

#[cfg(test)]
mod tests {
    use worth_proof::Performed;

    use super::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};

    mod owner {
        use super::*;

        worth_proof::authority_marker!(pub OwnerAuthority);

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OwnerBound {
            Entries,
            Bytes,
        }

        impl LimitDimension for OwnerBound {
            type Authority = OwnerAuthority;
        }

        pub fn refuse(
            dimension: OwnerBound,
            observed: u64,
            admitted: u64,
        ) -> ExhaustedLimit<OwnerBound> {
            let counts = LimitCounts::new(observed, admitted);
            ExhaustedLimit::refused(
                dimension,
                Performed::record(&OwnerAuthority::witness(), counts),
            )
        }
    }

    use owner::{refuse, OwnerBound};

    #[test]
    fn a_refusal_keeps_its_dimension_and_both_counts_in_place() {
        let limit = refuse(OwnerBound::Bytes, 65, 64);

        assert_eq!(limit.dimension(), OwnerBound::Bytes);
        assert_eq!(limit.counts(), LimitCounts::new(65, 64));
        assert_eq!((limit.observed(), limit.admitted()), (65, 64));
        assert_ne!(limit, refuse(OwnerBound::Bytes, 64, 65));
        assert_ne!(limit, refuse(OwnerBound::Entries, 65, 64));
    }

    #[test]
    fn counts_name_observed_then_admitted() {
        let counts = LimitCounts::new(3, 2);

        assert_eq!((counts.observed(), counts.admitted()), (3, 2));
    }

    #[test]
    fn the_refusal_action_has_no_value() {
        assert_eq!(core::mem::size_of::<BudgetRefused>(), 0);
        assert_eq!(
            core::mem::size_of::<ExhaustedLimit<OwnerBound>>(),
            core::mem::size_of::<(OwnerBound, LimitCounts)>()
        );
    }
}
