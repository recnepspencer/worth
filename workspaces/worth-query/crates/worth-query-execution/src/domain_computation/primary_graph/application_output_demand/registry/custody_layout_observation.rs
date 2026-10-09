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

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Reads the registry's retained indexed classes and its shared final-owner ledger.
    #[doc(hidden)]
    pub fn required_custody_breakdown_for_test(&self) -> Vec<(&'static str, usize, usize)> {
        let state = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.custody_breakdown_for_test()
    }
}

type Breakdown = Vec<(&'static str, usize, usize)>;
impl DemandRegistryState {
    fn custody_breakdown_for_test(&self) -> Breakdown {
        let members = self
            .required_keys
            .iter()
            .map(|key| required_members::member_bytes(key).unwrap())
            .sum();
        let slots: usize = self
            .records
            .values()
            .map(|record| record.settlements.capacity())
            .sum();
        let contexts: usize = self
            .records
            .values()
            .map(|record| record.settlements.len())
            .sum();
        let context_bytes: usize = self
            .records
            .values()
            .flat_map(|record| &record.settlements)
            .map(|(_, bytes)| *bytes)
            .sum();
        let prerequisites: usize = self
            .records
            .values()
            .map(|record| record.prerequisites.capacity())
            .sum();
        let pending: usize = self
            .records
            .values()
            .map(|record| record.pending_cleanup_key_bytes)
            .sum();
        let (postings, posting_bytes) = self.settlement_keys.custody_for_test();
        let mut rows = vec![
            ("members", self.required_keys.len(), members),
            ("settlement_slots", slots, slots * std::mem::size_of::<(Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>, usize)>()),
            ("execution_contexts", contexts, context_bytes),
            ("prerequisite_slots", prerequisites, prerequisites * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>()),
            ("settlement_postings", postings, posting_bytes),
            ("pending_cleanup_keys", 0, pending),
        ];
        let named: usize = rows.iter().map(|(_, _, bytes)| *bytes).sum();
        assert!(
            named <= self.required_reserved_bytes,
            "named classes fit the indexed ledger"
        );
        rows.push(("live_preparation", 0, self.required_reserved_bytes - named));
        rows.push((
            "shared_ready_source_continuation",
            0,
            self.required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
        ));
        rows
    }
}
