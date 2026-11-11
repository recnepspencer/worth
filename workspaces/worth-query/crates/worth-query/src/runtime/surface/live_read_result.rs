use crate::runtime::{WorthQueryIntentExecutionProvenance, WorthQueryLiveGraphReadAccessReceipt};

use super::super::WorthQueryIntentDecisionTraceEnvelope;
use super::WorthQueryLiveReadReceipt;

#[derive(Clone)]
pub struct WorthQueryLiveReadResult {
    rows: Vec<crate::memory_workspace::WorthQueryEntity>,
    maintenance_source_rows: Option<Vec<crate::memory_workspace::WorthQueryEntity>>,
    receipt: WorthQueryLiveReadReceipt,
}

impl std::fmt::Debug for WorthQueryLiveReadResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryLiveReadResult")
            .field("rows", &self.rows)
            .field("receipt", &self.receipt)
            .finish()
    }
}

impl PartialEq for WorthQueryLiveReadResult {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.receipt == other.receipt
    }
}

impl WorthQueryLiveReadResult {
    pub fn rows(&self) -> &[crate::memory_workspace::WorthQueryEntity] {
        &self.rows
    }

    pub fn receipt(&self) -> &WorthQueryLiveReadReceipt {
        &self.receipt
    }

    pub(crate) fn maintenance_source_rows(&self) -> &[crate::memory_workspace::WorthQueryEntity] {
        self.maintenance_source_rows
            .as_deref()
            .unwrap_or(&self.rows)
    }

    pub fn live_graph_read_access(&self) -> Option<&WorthQueryLiveGraphReadAccessReceipt> {
        self.receipt.live_graph_read_access()
    }

    pub(in crate::runtime) fn new(
        rows: Vec<crate::memory_workspace::WorthQueryEntity>,
        receipt: WorthQueryLiveReadReceipt,
    ) -> Self {
        Self {
            rows,
            maintenance_source_rows: None,
            receipt,
        }
    }

    pub(in crate::runtime) fn new_with_source_rows(
        rows: Vec<crate::memory_workspace::WorthQueryEntity>,
        maintenance_source_rows: Vec<crate::memory_workspace::WorthQueryEntity>,
        receipt: WorthQueryLiveReadReceipt,
    ) -> Self {
        Self {
            rows,
            maintenance_source_rows: Some(maintenance_source_rows),
            receipt,
        }
    }

    pub(in crate::runtime) fn attach_intent_admission_evidence(
        &mut self,
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
        execution_provenance: WorthQueryIntentExecutionProvenance,
    ) {
        self.receipt.decision_trace_envelope = Some(decision_trace_envelope);
        self.receipt.execution_provenance = Some(execution_provenance);
    }

    pub(in crate::runtime) fn attach_live_graph_read_access(
        &mut self,
        receipt: WorthQueryLiveGraphReadAccessReceipt,
    ) {
        self.receipt.live_graph_read_access = Some(receipt);
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        rows: Vec<crate::memory_workspace::WorthQueryEntity>,
        receipt: WorthQueryLiveReadReceipt,
    ) -> Self {
        Self {
            rows,
            maintenance_source_rows: None,
            receipt,
        }
    }
}
#[cfg(test)]
#[path = "live_read_result_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
