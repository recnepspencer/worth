use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SIGNAL_BRANCH_CELL_INCARNATION: AtomicU64 = AtomicU64::new(1);

/// Owner-issued identity for one installed branch-cell lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch) struct SignalBranchCellIncarnation(NonZeroU64);

impl SignalBranchCellIncarnation {
    pub(super) fn issue() -> Self {
        let identity = NEXT_SIGNAL_BRANCH_CELL_INCARNATION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("Signal branch cell incarnation identity exhausted");
        Self(NonZeroU64::new(identity).expect("cell incarnation identities start at one"))
    }

    pub(super) const fn get(self) -> u64 {
        self.0.get()
    }
}
