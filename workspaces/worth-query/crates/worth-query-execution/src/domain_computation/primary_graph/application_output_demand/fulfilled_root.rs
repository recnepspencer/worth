//! Retrospective custody of the root that fulfilled a performed source.
use super::WorthQueryOutputSettlementPosture;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use std::sync::Arc;

/// The actual required demand fulfilled the source's obligation at this output
/// occurrence. Reuse/restoration does not assert producer re-execution.
/// This owns readonly evidence, never currentness or publication permission.
/// Borrow the facts carried by an actual completed required root:
/// ```
/// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryFulfilledRequiredRoot};
/// fn inspect(fact: &WorthQueryFulfilledRequiredRoot, source: &WorthQueryApplicationCommitReceipt) {
///     let _ = (fact.source_commit(), fact.output_commit(), fact.posture());
///     let _ = fact.matches_source(source);
/// }
/// ```
/// A caller cannot construct fulfillment from descriptive identities:
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryFulfilledRequiredRoot;
/// fn forge() -> WorthQueryFulfilledRequiredRoot {
///     WorthQueryFulfilledRequiredRoot {
///         source: panic!(), output: panic!(), source_intent: panic!(),
///         posture: panic!(), _memory: None,
///     }
/// }
/// ```
/// The fact cannot become an ordinary query selector:
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryFulfilledRequiredRoot;
/// fn select(fact: &WorthQueryFulfilledRequiredRoot) {
///     let _ = fact.select();
/// }
/// ```
pub struct WorthQueryFulfilledRequiredRoot {
    pub(super) source: worth_runtime_world::facade::ProductBranchObservation,
    pub(super) output: Arc<WorthQueryApplicationReadObservation>,
    pub(super) source_intent: ([u8; 32], String),
    pub(super) posture: WorthQueryOutputSettlementPosture,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryFulfilledRequiredRoot {
    pub(in crate::domain_computation::primary_graph) fn retained_output(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
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
