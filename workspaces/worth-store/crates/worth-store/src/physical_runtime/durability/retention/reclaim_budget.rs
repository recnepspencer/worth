//! Bounded incremental reclamation (`D5`, `ReclaimBudget`).
//!
//! Reclamation runs as maintenance passes off the foreground commit path.
//! Each pass retires at most the budget slice, admitted artifact by artifact
//! through the verified `ReclamationPlanValidator`, so no pass scans or
//! frees unbounded retained state.

use super::PhysicalRetirementDenial;
use crate::reclamation_plan_validator::{budget_admits, ReclaimBudget};

/// The bounded slice one incremental reclamation pass may retire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReclaimBudget {
    max_artifacts: u64,
    max_bytes: u64,
}

impl PhysicalReclaimBudget {
    pub const fn new(max_artifacts: u64, max_bytes: u64) -> Self {
        Self {
            max_artifacts,
            max_bytes,
        }
    }

    pub const fn max_artifacts(self) -> u64 {
        self.max_artifacts
    }

    pub const fn max_bytes(self) -> u64 {
        self.max_bytes
    }

    /// Whether one more artifact of `next_bytes` fits after `pass`.
    pub(in crate::physical_runtime) fn admits(
        self,
        pass: &PhysicalReclaimPass,
        next_bytes: u64,
    ) -> bool {
        budget_admits(
            pass.retired_artifacts,
            pass.retired_bytes,
            next_bytes,
            ReclaimBudget {
                max_artifacts: self.max_artifacts,
                max_bytes: self.max_bytes,
            },
        )
    }
}

/// Why a reclamation pass stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalReclaimPassEnd {
    /// No displaced artifact awaits retirement.
    Drained,
    /// The next artifact does not fit the remaining budget.
    BudgetExhausted,
    /// The next artifact is protected, uncovered, unresolved or failed.
    Denied(PhysicalRetirementDenial),
}

/// What one bounded reclamation pass retired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReclaimPass {
    pub retired_artifacts: u64,
    pub retired_bytes: u64,
    pub end: PhysicalReclaimPassEnd,
}
