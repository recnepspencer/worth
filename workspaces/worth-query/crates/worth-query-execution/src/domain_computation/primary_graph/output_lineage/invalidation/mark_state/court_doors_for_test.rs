//! Sealed test doors over this module's private members.
use super::*;

impl MarkState {
    #[doc(hidden)]
    pub fn posting_bindings_for_test(
        &self,
    ) -> &OrdMap<u64, Arc<super::super::settlement::PostingBindings>> {
        &self.posting_bindings
    }
}

impl MarkState {
    #[doc(hidden)]
    pub fn posting_count_for_test(&self) -> &usize {
        &self.posting_count
    }
}

impl MarkState {
    #[doc(hidden)]
    pub fn postings_for_test(&self) -> &OrdMap<Arc<FactPostingKey>, OrdSet<u64>> {
        &self.postings
    }
}

impl FactPosting {
    #[doc(hidden)]
    pub fn settlement_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.settlement
    }
}

impl MarkState {
    #[doc(hidden)]
    pub fn posting_count_mut_for_test(&mut self) -> &mut usize {
        &mut self.posting_count
    }
}

impl MarkState {
    #[doc(hidden)]
    pub fn initial_for_test() -> Self {
        Self::initial()
    }
}
