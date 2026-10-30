use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SIGNAL_BRANCH_CELL_INCARNATION: AtomicU64 = AtomicU64::new(1);

/// Owner-issued identity for one installed branch-cell lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch) struct SignalBranchCellIncarnation(NonZeroU64);

impl SignalBranchCellIncarnation {
    pub(super) fn issue() -> Self {
        let identity = {
            let update_counter = &NEXT_SIGNAL_BRANCH_CELL_INCARNATION;
            let mut current = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = current.checked_add(1) else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .expect("Signal branch cell incarnation identity exhausted");
        Self(NonZeroU64::new(identity).expect("cell incarnation identities start at one"))
    }

    pub(super) const fn get(self) -> u64 {
        self.0.get()
    }
}
