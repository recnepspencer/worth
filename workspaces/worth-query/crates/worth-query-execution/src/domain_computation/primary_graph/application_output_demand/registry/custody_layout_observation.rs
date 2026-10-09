//! One layout observation for the named owners beneath required custody.
use super::*;

/// Funded sizes of the required-row owners; observing them grants no custody.
#[doc(hidden)]
pub struct RequiredCustodyLayoutForTest {
    pub ready: usize,
    pub source: usize,
    pub context_without_producer: usize,
    pub member_without_producer: usize,
    pub outer_posting_node: usize,
    pub inner_posting_node: usize,
    pub posting: usize,
    pub settlement_slot: usize,
    pub prerequisite_slot: usize,
}
/// Reads owner sizes for independent required-custody formulas in tests.
#[doc(hidden)]
pub fn required_custody_layout_for_test() -> RequiredCustodyLayoutForTest {
    let (outer_posting_node, inner_posting_node, posting) = settlement_index::custody_layout();
    RequiredCustodyLayoutForTest {
        ready: ready_backing::PreparedReadyBacking::retained_bytes(),
        source: source_readmission::retained_bytes().unwrap(),
        context_without_producer: required_context::minimum_context_bytes(),
        member_without_producer: required_members::minimum_member_bytes().unwrap(),
        outer_posting_node, inner_posting_node, posting,
        settlement_slot: std::mem::size_of::<(Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>, usize)>(),
        prerequisite_slot: std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>(),
    }
}
