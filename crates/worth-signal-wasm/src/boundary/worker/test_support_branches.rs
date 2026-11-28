use wasm_bindgen::JsValue;

use worth_signal::facade::adapters::BranchStateProofReport;
use worth_signal::facade::history::{RuntimeBranch, RuntimeSnapshot};

use crate::runtime::summaries::{ReplaySummary, RuntimeSnapshotEnvelope};
use crate::runtime::worker_host::{
    WorkerApplyTransactionToBranchReceipt, WorkerApplyTransactionToBranchRequest,
    WorkerBranchBasisReceipt, WorkerBranchTruthEnvelope, WorkerCloseoutEffectBranchReceipt,
    WorkerCloseoutEffectBranchRequest, WorkerForkBranchReceipt, WorkerForkBranchRequest,
    WorkerRetireBranchReceipt, WorkerRetireBranchRequest, WorkerRetireBranchesReceipt,
    WorkerRetireBranchesRequest,
};

use super::SignalWorkerRuntime;

impl SignalWorkerRuntime {
    pub(crate) fn current_branch_for_test(&self) -> Result<RuntimeBranch, JsValue> {
        Ok(self.shell.borrow().current_branch())
    }

    pub(crate) fn branches_for_test(&self) -> Result<Vec<RuntimeBranch>, JsValue> {
        Ok(self.shell.borrow().branches())
    }

    pub(crate) fn replay_for_branch_for_test(
        &self,
        branch_id: u64,
    ) -> Result<ReplaySummary, JsValue> {
        self.shell
            .borrow_mut()
            .replay_for_branch(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn branch_snapshot_id_for_test(&self, branch_id: u64) -> Result<u64, JsValue> {
        self.shell
            .borrow_mut()
            .branch_snapshot_id(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn branch_snapshot_envelope_for_test(
        &self,
        branch_id: u64,
    ) -> Result<RuntimeSnapshotEnvelope, JsValue> {
        self.shell
            .borrow_mut()
            .branch_snapshot_envelope(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn branch_snapshot_for_test(
        &self,
        branch_id: u64,
    ) -> Result<RuntimeSnapshot, JsValue> {
        self.shell
            .borrow_mut()
            .branch_snapshot(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn create_branch_for_test(&self, name: String) -> Result<RuntimeBranch, JsValue> {
        self.shell
            .borrow_mut()
            .create_branch(name)
            .map_err(JsValue::from)
    }

    pub(crate) fn worker_branch_basis_for_test(
        &self,
        branch_id: u64,
    ) -> Result<WorkerBranchBasisReceipt, JsValue> {
        self.shell
            .borrow()
            .worker_branch_basis(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn fork_worker_branch_for_test(
        &self,
        request: WorkerForkBranchRequest,
    ) -> Result<WorkerForkBranchReceipt, JsValue> {
        self.shell
            .borrow_mut()
            .fork_worker_branch(request)
            .map_err(JsValue::from)
    }

    pub(crate) fn apply_transaction_to_worker_branch_for_test(
        &self,
        request: WorkerApplyTransactionToBranchRequest,
    ) -> Result<WorkerApplyTransactionToBranchReceipt, JsValue> {
        self.shell
            .borrow_mut()
            .apply_transaction_to_worker_branch(request)
            .map_err(JsValue::from)
    }

    pub(crate) fn retire_worker_branch_for_test(
        &self,
        request: WorkerRetireBranchRequest,
    ) -> Result<WorkerRetireBranchReceipt, JsValue> {
        self.shell
            .borrow_mut()
            .retire_worker_branch(request)
            .map_err(JsValue::from)
    }

    pub(crate) fn retire_worker_branches_for_test(
        &self,
        request: WorkerRetireBranchesRequest,
    ) -> Result<WorkerRetireBranchesReceipt, JsValue> {
        self.shell
            .borrow_mut()
            .retire_worker_branches(request)
            .map_err(JsValue::from)
    }

    pub(crate) fn closeout_worker_effect_branch_for_test(
        &self,
        request: WorkerCloseoutEffectBranchRequest,
    ) -> Result<WorkerCloseoutEffectBranchReceipt, JsValue> {
        self.shell
            .borrow_mut()
            .closeout_worker_effect_branch(request)
            .map_err(JsValue::from)
    }

    pub(crate) fn switch_branch_for_test(
        &self,
        branch_id: u64,
    ) -> Result<WorkerBranchTruthEnvelope, JsValue> {
        self.shell
            .borrow_mut()
            .switch_branch(branch_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn restore_branch_snapshot_for_test(
        &self,
        branch_id: u64,
        snapshot: RuntimeSnapshot,
    ) -> Result<WorkerBranchTruthEnvelope, JsValue> {
        self.shell
            .borrow_mut()
            .restore_branch_snapshot(branch_id, snapshot)
            .map_err(JsValue::from)
    }

    pub(crate) fn restore_branch_snapshot_by_id_for_test(
        &self,
        branch_id: u64,
        snapshot_id: u64,
    ) -> Result<WorkerBranchTruthEnvelope, JsValue> {
        self.shell
            .borrow_mut()
            .restore_branch_snapshot_by_id(branch_id, snapshot_id)
            .map_err(JsValue::from)
    }

    pub(crate) fn branch_state_proof_for_test(
        &self,
        branch_id: u64,
    ) -> Result<BranchStateProofReport, JsValue> {
        self.shell
            .borrow()
            .branch_state_proof(branch_id)
            .map_err(JsValue::from)
    }
}
