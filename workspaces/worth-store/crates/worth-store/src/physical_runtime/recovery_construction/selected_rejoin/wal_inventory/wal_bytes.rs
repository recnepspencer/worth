//! The selected WAL inventory's own byte budget: the only place that mints its
//! read grant and its limit. A leaf module, because the authority's declaring
//! module and its descendants can mint.

use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};
use worth_proof::Performed;
use worth_store_physical_backend::{GrantOverrun, ReadGrant};

worth_foundational::limit_authority!(pub SelectedWalInventoryBudgetAuthority);

/// What one complete selected WAL inventory ran out of: the encoded WAL bytes
/// it may read and retain in all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedWalInventoryBound {
    WalBytes,
}

impl LimitDimension for SelectedWalInventoryBound {
    type Authority = SelectedWalInventoryBudgetAuthority;
}

/// A selected WAL inventory stopped before its first effect at the bytes its
/// budget admits. It says nothing about the media: `observed` is the WAL bytes
/// through the file that crossed the budget, each counted at its real length.
pub type ExceededSelectedWalInventoryBound = ExhaustedLimit<SelectedWalInventoryBound>;

/// The whole one complete inventory read may spend, granted to that read so
/// the backend refuses the file that crosses it before reading it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectedWalInventoryBudget {
    whole: u64,
}

impl SelectedWalInventoryBudget {
    /// The inventory every selected rejoin reads: `MAX_WAL_BYTES`, the bytes
    /// the retained-memory envelope budgets for its frames.
    pub(super) const fn complete() -> Self {
        Self {
            whole: super::MAX_WAL_BYTES,
        }
    }

    /// A test's narrower whole, to cross it with a real WAL file.
    #[cfg(all(test, windows))]
    pub(super) const fn with_whole_for_test(whole: u64) -> Self {
        Self { whole }
    }

    /// The whole, granted to the one read of the complete inventory.
    pub(super) fn grant(&self) -> ReadGrant<SelectedWalInventoryBound> {
        ReadGrant::granted(
            SelectedWalInventoryBound::WalBytes,
            Performed::record(&SelectedWalInventoryBudgetAuthority::witness(), self.whole),
        )
    }

    /// The read past the whole, as this budget's limit: the WAL bytes through
    /// the crossing file past the bytes granted.
    pub(super) fn refuse(
        overrun: GrantOverrun<SelectedWalInventoryBound>,
    ) -> ExceededSelectedWalInventoryBound {
        SelectedWalInventoryBudgetAuthority::refuse(
            overrun.dimension(),
            LimitCounts::new(overrun.length(), overrun.granted()),
        )
    }
}
