//! Sealed test doors over this module's private members.
use super::*;

impl ComputationCallCharge {
    #[doc(hidden)]
    pub fn accounting_for_test(&self) -> &Option<ComputationCallAccounting> {
        &self.accounting
    }
}

impl ComputationCallCharge {
    #[doc(hidden)]
    pub fn reached_mut_for_test(&mut self) -> &mut Arc<[EntityId]> {
        &mut self.reached
    }
}

impl ComputationCallAccounting {
    #[doc(hidden)]
    pub fn consumed_for_test(&self) -> &usize {
        &self.consumed
    }
}

impl ComputationCallAccounting {
    #[doc(hidden)]
    pub fn demand_for_test(&self) -> &usize {
        &self.demand
    }
}

impl ComputationCallCharge {
    #[doc(hidden)]
    pub fn new_for_test(
        accounting: Option<ComputationCallAccounting>,
        reached: Arc<[EntityId]>,
    ) -> Self {
        Self {
            accounting,
            reached,
        }
    }
}

impl ComputationCallAccounting {
    #[doc(hidden)]
    pub fn new_for_test(demand: usize, consumed: usize) -> Self {
        Self { demand, consumed }
    }
}
