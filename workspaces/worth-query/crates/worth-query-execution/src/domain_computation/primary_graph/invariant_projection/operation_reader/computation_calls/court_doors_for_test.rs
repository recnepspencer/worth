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
