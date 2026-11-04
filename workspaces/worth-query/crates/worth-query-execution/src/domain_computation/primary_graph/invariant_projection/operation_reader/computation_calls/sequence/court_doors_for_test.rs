//! Sealed test doors over this module's private members.
use super::*;

impl CallSummary {
    #[doc(hidden)]
    pub fn accounting_for_test(&self) -> &Option<ComputationCallAccounting> {
        &self.accounting
    }
}

impl CallSummary {
    #[doc(hidden)]
    pub fn declared_for_test(&self) -> &Option<u64> {
        &self.declared
    }
}

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn shape_for_test(&self) -> &Arc<CallShape> {
        &self.shape
    }
}
