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

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn visit_for_test<E>(
        &self,
        range: Range<usize>,
        marked: &BTreeSet<usize>,
        visitor: &mut impl FnMut(CallVisit<'_>) -> Result<bool, E>,
    ) -> Result<(), E> {
        Self::visit(self, range, marked, visitor)
    }
}

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn bytes_for_test(&self) -> Option<u64> {
        Self::bytes(self)
    }
}

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn prefix_for_test(&self) -> usize {
        Self::prefix(self)
    }
}
