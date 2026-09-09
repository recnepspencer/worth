use serde::{Deserialize, Serialize};

use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

/// Owner-wide quotas for managed temporal partitions, fixed at owner sealing.
/// Representation bytes also consume the shared conditional retention ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalConditionalTemporalBudget {
    /// Capacity remains reserved while a retired partition's capability or an
    /// in-flight cell still retains its backing allocation.
    pub maximum_live_partitions: usize,
    /// Sum of the W bounds reserved by those partitions, independent of how
    /// many scheduled or ready wakes currently occupy them.
    pub maximum_reserved_active_wakes: usize,
}

impl SignalConditionalTemporalBudget {
    pub(super) const PRESET: Self = Self {
        maximum_live_partitions: 128,
        maximum_reserved_active_wakes: 16_384,
    };

    pub(super) const fn is_valid(self) -> bool {
        self.maximum_live_partitions != 0 && self.maximum_reserved_active_wakes != 0
    }
}

impl SignalRuntimePolicy {
    pub fn with_conditional_temporal_budget(
        mut self,
        budget: SignalConditionalTemporalBudget,
    ) -> Self {
        self.conditional_temporal_budget = budget;
        self
    }
}

impl ResolvedSignalRuntimePolicy {
    pub const fn conditional_temporal_budget(&self) -> SignalConditionalTemporalBudget {
        self.conditional_temporal_budget
    }
}

impl InstalledSignalRuntimePolicy {
    pub const fn conditional_temporal_budget(&self) -> SignalConditionalTemporalBudget {
        self.resolved().conditional_temporal_budget()
    }
}

#[cfg(test)]
mod tests;
