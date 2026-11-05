//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

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

impl MarkState {
    #[doc(hidden)]
    pub fn settlements_for_test(
        &self,
    ) -> &OrdMap<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>> {
        &self.settlements
    }
}

impl SettlementMarks {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &super::super::super::RetainedSourceFacts {
        &self.facts
    }
}

impl SettlementMarks {
    #[doc(hidden)]
    pub fn posting_projection_for_test(&self) -> &Arc<super::super::settlement::PostingProjection> {
        &self.posting_projection
    }
}

impl FactPosting {
    #[doc(hidden)]
    pub fn new_for_test(settlement: Arc<RecordedSettlementIdentity>, ordinal: usize) -> Self {
        Self {
            settlement,
            ordinal,
        }
    }
}
