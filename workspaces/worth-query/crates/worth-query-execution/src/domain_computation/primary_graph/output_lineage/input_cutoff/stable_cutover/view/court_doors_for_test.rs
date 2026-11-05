//! Sealed test doors over this module's private members.
use super::*;

impl PublishedStableLineage {
    #[doc(hidden)]
    pub fn exact_settlement_for_test(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity>{
        Self::exact_settlement(self)
    }
}
