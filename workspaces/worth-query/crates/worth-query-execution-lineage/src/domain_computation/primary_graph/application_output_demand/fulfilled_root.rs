//! Retrospective custody of the root that fulfilled a performed source.
use super::WorthQueryOutputSettlementPosture;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use std::sync::Arc;

/// The actual required demand fulfilled the source's obligation at this output
/// occurrence. Reuse/restoration does not assert producer re-execution.
/// This owns readonly evidence, never currentness or publication permission.
/// Borrow the facts carried by an actual completed required root:
/// A caller cannot construct fulfillment from descriptive identities:
/// The fact cannot become an ordinary query selector:
pub struct WorthQueryFulfilledRequiredRoot {
    pub(crate) source: worth_runtime_world::facade::ProductBranchObservation,
    pub(crate) output: Arc<WorthQueryApplicationReadObservation>,
    pub(crate) source_intent: ([u8; 32], String),
    pub(crate) posture: WorthQueryOutputSettlementPosture,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryFulfilledRequiredRoot {
    pub fn retained_output(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.output
    }
    pub fn source_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.source.selected_commit()
    }
    pub fn output_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.output.selected_commit()
    }
    pub fn posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }
    pub fn matches_source(&self, source: impl Into<super::WorthQueryOutputSourceEvidence>) -> bool {
        let source = source.into();
        source.subject() == self.source_intent && source.is_selected_at(&self.source)
    }
}
