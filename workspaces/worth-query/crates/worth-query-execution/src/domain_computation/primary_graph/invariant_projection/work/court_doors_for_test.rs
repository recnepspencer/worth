//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn bounded_for_test(maximum: usize) -> Self {
        Self::bounded(maximum)
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn can_afford_for_test(&mut self, maximum_work: usize) -> bool {
        Self::can_afford(self, maximum_work)
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn carry_for_test(&mut self, demand: usize, consumed: usize) -> bool {
        Self::carry(self, demand, consumed)
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn consume_for_test(&mut self, actual_work: usize) -> bool {
        Self::consume(self, actual_work)
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn exceeded_for_test(&self) -> bool {
        Self::exceeded(self)
    }
}

impl WorthQueryInvariantProjectionWorkBudget {
    #[doc(hidden)]
    pub fn remaining_for_test(&self) -> Option<usize> {
        Self::remaining(self)
    }
}
