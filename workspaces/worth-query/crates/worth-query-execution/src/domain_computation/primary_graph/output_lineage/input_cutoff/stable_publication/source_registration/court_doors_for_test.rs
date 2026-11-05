//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl<'selected> StableEqualityConsequence<'selected> {
    #[doc(hidden)]
    pub fn native_actor_fixture_for_test(
        predecessor: Arc<RecordedSettlementIdentity>,
        successor: Arc<RecordedSettlementIdentity>,
        selected: &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Self {
        Self::native_actor_fixture(predecessor, successor, selected)
    }
}
