//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl PostingBindings {
    #[doc(hidden)]
    pub fn settlements_for_test(&self) -> Option<&SettlementPostingBindings> {
        Self::settlements(self)
    }
}

impl SettlementPostingBindings {
    #[doc(hidden)]
    pub fn settlements_for_test(&self) -> &OrdSet<Arc<RecordedSettlementIdentity>> {
        &self.settlements
    }
}
