use serde::{Deserialize, Serialize};

use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

/// Optional host quotas for owner-managed derived evaluation. These limits do not
/// authorize execution; the owner must reserve actual storage and work first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalConditionalEvaluationBudget {
    /// Optional owner-wide slot ceiling. All live branches must agree at sealing.
    #[serde(deserialize_with = "Option::deserialize")]
    pub maximum_retained_slots: Option<usize>,
    /// Shared Signal conditional-service representation bytes, including live
    /// old/draft/evidence custody and managed temporal partitions and results.
    /// Other owners must account their own storage under their own ceilings.
    /// This is not an allocator or process-RSS limit.
    /// None omits this host quota; actual backing remains reserved and counted.
    /// All live branches must agree on this policy at sealing.
    #[serde(deserialize_with = "Option::deserialize")]
    pub maximum_retained_bytes: Option<u64>,
    /// Optional caller bound per selected-definition attempt; never replenished by a writer.
    pub maximum_attempt_visits: Option<usize>,
}

impl SignalConditionalEvaluationBudget {
    pub(super) const PRESET: Self = Self {
        maximum_retained_slots: None,
        maximum_retained_bytes: None,
        maximum_attempt_visits: None,
    };

    pub const fn development() -> Self {
        Self::PRESET
    }

    pub(super) const fn is_valid(self) -> bool {
        !matches!(self.maximum_retained_slots, Some(0))
            && !matches!(self.maximum_retained_bytes, Some(0))
            && match self.maximum_attempt_visits {
                None => true,
                Some(maximum) => maximum != 0,
            }
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
