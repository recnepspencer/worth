use serde::{Deserialize, Serialize};

use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

/// Requested bounds for owner-managed derived evaluation. These limits do not
/// authorize execution; the owner must reserve actual storage and work first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalConditionalEvaluationBudget {
    /// Fixed owner-wide ceiling at sealing. All live branches must agree.
    pub maximum_retained_slots: usize,
    /// Shared Signal conditional-service representation bytes, including live
    /// old/draft/evidence custody and managed temporal partitions and results.
    /// Other owners must account their own storage under their own ceilings.
    /// This is not an allocator or process-RSS limit.
    /// All live branches must agree on this owner-wide ceiling at sealing.
    pub maximum_retained_bytes: u64,
    /// Per selected-definition attempt; never replenished by a storage writer.
    pub maximum_attempt_visits: usize,
}

impl SignalConditionalEvaluationBudget {
    pub(super) const PRESET: Self = Self {
        maximum_retained_slots: 128,
        maximum_retained_bytes: 512 * 1024 * 1024,
        maximum_attempt_visits: 8_000_000,
    };

    pub const fn development() -> Self {
        Self::PRESET
    }

    pub(super) const fn is_valid(self) -> bool {
        self.maximum_retained_slots != 0
            && self.maximum_retained_bytes != 0
            && self.maximum_attempt_visits != 0
    }
}

impl SignalRuntimePolicy {
    pub fn with_conditional_evaluation_budget(
        mut self,
        budget: SignalConditionalEvaluationBudget,
    ) -> Self {
        self.conditional_evaluation_budget = budget;
        self
    }
}

impl ResolvedSignalRuntimePolicy {
    pub const fn conditional_evaluation_budget(&self) -> SignalConditionalEvaluationBudget {
        self.conditional_evaluation_budget
    }
}

impl InstalledSignalRuntimePolicy {
    pub const fn conditional_evaluation_budget(&self) -> SignalConditionalEvaluationBudget {
        self.resolved().conditional_evaluation_budget()
    }
}

#[cfg(test)]
mod tests;
