use crate::runtime::{
    UiAllocationFrameReplacementTransition, UiCommittedAllocationActivationCounters,
    UiCommittedAllocationReplan, UiScrollCatalogSwapEvidence, UiScrollOwnerCatalogReceipt,
    WorthUiActivationGateReceipt, WorthUiPriorValidPlanObservation, WorthUiRuntimeFrameEpoch,
};

#[derive(Debug, PartialEq)]
pub struct WorthUiPlanSwapReceipt {
    attempt_identity_digest: u64,
    committed_row_count: usize,
    previous_active_artifact_digest: u64,
    previous_active_plan_digest: u64,
    previous_active_snapshot_digest: u64,
    next_active_artifact_digest: u64,
    next_active_plan_digest: u64,
    next_active_snapshot_digest: u64,
    activation_gate_receipt: WorthUiActivationGateReceipt,
    prior_valid_plan: WorthUiPriorValidPlanObservation,
    counters: UiCommittedAllocationActivationCounters,
    allocation_frame_replacement: UiAllocationFrameReplacementTransition,
    scroll_catalog_evidence: UiScrollCatalogSwapEvidence,
    committed_allocation: UiCommittedAllocationReplan,
    structural_reuse: crate::runtime::WorthUiPlanRegionalEvidence,
}

pub(crate) struct WorthUiPlanSwapReceiptDraft {
    pub(crate) attempt_identity_digest: u64,
    pub(crate) committed_row_count: usize,
    pub(crate) previous_active_artifact_digest: u64,
    pub(crate) previous_active_plan_digest: u64,
    pub(crate) previous_active_snapshot_digest: u64,
    pub(crate) next_active_artifact_digest: u64,
    pub(crate) next_active_plan_digest: u64,
    pub(crate) next_active_snapshot_digest: u64,
    pub(crate) activation_gate_receipt: WorthUiActivationGateReceipt,
    pub(crate) prior_valid_plan: WorthUiPriorValidPlanObservation,
    pub(crate) counters: UiCommittedAllocationActivationCounters,
    pub(crate) scroll_catalog_evidence: UiScrollCatalogSwapEvidence,
    pub(crate) committed_allocation: UiCommittedAllocationReplan,
    pub(crate) structural_reuse: crate::runtime::WorthUiPlanRegionalEvidence,
}

impl WorthUiPlanSwapReceiptDraft {
    pub(crate) fn finish(
        self,
        allocation_frame_replacement: UiAllocationFrameReplacementTransition,
    ) -> WorthUiPlanSwapReceipt {
        WorthUiPlanSwapReceipt {
            attempt_identity_digest: self.attempt_identity_digest,
            committed_row_count: self.committed_row_count,
            previous_active_artifact_digest: self.previous_active_artifact_digest,
            previous_active_plan_digest: self.previous_active_plan_digest,
            previous_active_snapshot_digest: self.previous_active_snapshot_digest,
            next_active_artifact_digest: self.next_active_artifact_digest,
            next_active_plan_digest: self.next_active_plan_digest,
            next_active_snapshot_digest: self.next_active_snapshot_digest,
            activation_gate_receipt: self.activation_gate_receipt,
            prior_valid_plan: self.prior_valid_plan,
            counters: self.counters,
            allocation_frame_replacement,
            scroll_catalog_evidence: self.scroll_catalog_evidence,
            committed_allocation: self.committed_allocation,
            structural_reuse: self.structural_reuse,
        }
    }
}

impl WorthUiPlanSwapReceipt {
    pub fn inspection(&self) -> super::UiCommittedAllocationActivationInspection {
        super::UiCommittedAllocationActivationInspection::committed(self)
    }
    pub fn attempt_identity_digest(&self) -> u64 {
        self.attempt_identity_digest
    }
    pub fn committed_row_count(&self) -> usize {
        self.committed_row_count
    }
    #[cfg(test)]
    pub(crate) fn with_corrupted_previous_artifact_digest_for_test(mut self) -> Self {
        self.previous_active_artifact_digest = self.previous_active_artifact_digest.wrapping_add(1);
        self
    }
    #[cfg(test)]
    pub(crate) fn allocation_frame_replacement(&self) -> &UiAllocationFrameReplacementTransition {
        &self.allocation_frame_replacement
    }
    pub fn previous_active_artifact_digest(&self) -> u64 {
        self.previous_active_artifact_digest
    }
    pub fn previous_active_plan_digest(&self) -> u64 {
        self.previous_active_plan_digest
    }
    pub fn previous_active_snapshot_digest(&self) -> u64 {
        self.previous_active_snapshot_digest
    }
    pub fn next_active_artifact_digest(&self) -> u64 {
        self.next_active_artifact_digest
    }
    pub fn next_active_plan_digest(&self) -> u64 {
        self.next_active_plan_digest
    }
    pub fn next_active_snapshot_digest(&self) -> u64 {
        self.next_active_snapshot_digest
    }
    pub fn activation_gate_receipt(&self) -> WorthUiActivationGateReceipt {
        self.activation_gate_receipt
    }
    pub fn prior_valid_plan(&self) -> WorthUiPriorValidPlanObservation {
        self.prior_valid_plan
    }
    pub fn readiness_frame_epoch(&self) -> WorthUiRuntimeFrameEpoch {
        self.activation_gate_receipt.readiness_frame_epoch()
    }
    pub fn boundary_frame_epoch(&self) -> WorthUiRuntimeFrameEpoch {
        self.activation_gate_receipt.boundary_frame_epoch()
    }
    pub fn node_classification_count(&self) -> usize {
        self.activation_gate_receipt.node_classification_count()
    }
    pub fn lane_changed_node_count(&self) -> usize {
        self.activation_gate_receipt.lane_changed_node_count()
    }
    pub fn reconciliation_basis_digest(&self) -> u64 {
        self.activation_gate_receipt.reconciliation_basis_digest()
    }
    pub fn reconciliation_receipt_count(&self) -> usize {
        self.activation_gate_receipt.reconciliation_receipt_count()
    }
    pub fn query_rebind_basis_digest(&self) -> u64 {
        self.activation_gate_receipt.query_rebind_basis_digest()
    }
    pub fn query_rebind_entry_count(&self) -> usize {
        self.activation_gate_receipt.query_rebind_entry_count()
    }
    pub fn query_rebind_denied_count(&self) -> usize {
        self.activation_gate_receipt.query_rebind_denied_count()
    }
    pub fn lane_parity_semantic_reference_digest(&self) -> Option<u64> {
        self.activation_gate_receipt
            .lane_parity_semantic_reference_digest()
    }
    pub fn counters(&self) -> UiCommittedAllocationActivationCounters {
        self.counters
    }
    pub fn scroll_owner_catalog(&self) -> Option<UiScrollOwnerCatalogReceipt> {
        match &self.scroll_catalog_evidence {
            UiScrollCatalogSwapEvidence::Prepared(receipt) => Some(receipt.clone()),
            UiScrollCatalogSwapEvidence::Denied(_) => None,
        }
    }
    pub fn scroll_catalog_evidence(&self) -> UiScrollCatalogSwapEvidence {
        self.scroll_catalog_evidence.clone()
    }
    pub fn committed_allocation(&self) -> &UiCommittedAllocationReplan {
        &self.committed_allocation
    }

    pub fn structural_reuse(&self) -> &crate::runtime::WorthUiPlanRegionalEvidence {
        &self.structural_reuse
    }
}
