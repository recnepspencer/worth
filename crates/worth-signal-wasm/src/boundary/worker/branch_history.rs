use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use worth_signal::facade::history::RuntimeSnapshot;

use crate::boundary::restore_tokens::{
    load_snapshot, load_snapshot_envelope, store_snapshot, store_snapshot_envelope,
};
use crate::boundary::serde::{from_js, from_json_wire, to_js, to_js_structured, to_json_wire};
use crate::runtime::core::MergePolicyPreviewRequest;
use crate::runtime::summaries::RuntimeSnapshotEnvelope;
use crate::runtime::worker_host::{
    WorkerApplyTransactionToBranchRequest, WorkerCloseoutEffectBranchRequest,
    WorkerForkBranchRequest, WorkerRetireBranchRequest, WorkerRetireBranchesRequest,
};

use super::SignalWorkerRuntime;

#[wasm_bindgen]
impl SignalWorkerRuntime {
    #[wasm_bindgen(js_name = restoreSnapshotEnvelope)]
    pub fn restore_snapshot_envelope(&self, snapshot: JsValue) -> Result<JsValue, JsValue> {
        let snapshot: RuntimeSnapshotEnvelope = from_js(snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_snapshot_for_test(snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreSnapshotEnvelopeWire)]
    pub fn restore_snapshot_envelope_wire(&self, snapshot: String) -> Result<JsValue, JsValue> {
        let snapshot = load_snapshot_envelope(&snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_snapshot_for_test(snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreSnapshotEnvelopePortableWire)]
    pub fn restore_snapshot_envelope_portable_wire(
        &self,
        snapshot: String,
    ) -> Result<JsValue, JsValue> {
        let snapshot: RuntimeSnapshotEnvelope = from_json_wire(&snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_snapshot_for_test(snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = currentBranch)]
    pub fn current_branch(&self) -> Result<JsValue, JsValue> {
        to_js(&self.current_branch_for_test()?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = branches)]
    pub fn branches(&self) -> Result<JsValue, JsValue> {
        to_js(&self.branches_for_test()?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = createBranch)]
    pub fn create_branch(&self, name: String) -> Result<JsValue, JsValue> {
        to_js(&self.create_branch_for_test(name)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = workerBranchBasis)]
    pub fn worker_branch_basis(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        to_js(&self.worker_branch_basis_for_test(branch_id)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = forkBranch)]
    pub fn fork_worker_branch(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerForkBranchRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.fork_worker_branch_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = applyTransactionToBranch)]
    pub fn apply_transaction_to_worker_branch(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerApplyTransactionToBranchRequest =
            from_js(request).map_err(JsValue::from)?;
        to_js(&self.apply_transaction_to_worker_branch_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = retireBranch)]
    pub fn retire_worker_branch(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerRetireBranchRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.retire_worker_branch_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = retireBranches)]
    pub fn retire_worker_branches(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerRetireBranchesRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.retire_worker_branches_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = closeoutEffectBranch)]
    pub fn closeout_worker_effect_branch(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: WorkerCloseoutEffectBranchRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.closeout_worker_effect_branch_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = switchBranch)]
    pub fn switch_branch(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        let envelope = self.switch_branch_for_test(branch_id)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = replayForBranch)]
    pub fn replay_for_branch(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        to_js(&self.replay_for_branch_for_test(branch_id)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = branchSnapshotId)]
    pub fn branch_snapshot_id(&self, branch_id: u64) -> Result<u64, JsValue> {
        self.branch_snapshot_id_for_test(branch_id)
    }

    #[wasm_bindgen(js_name = branchSnapshotEnvelope)]
    pub fn branch_snapshot_envelope(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        to_js_structured(&self.branch_snapshot_envelope_for_test(branch_id)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = branchSnapshotEnvelopeArtifact)]
    pub fn branch_snapshot_envelope_artifact(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        let snapshot = self.branch_snapshot_envelope_for_test(branch_id)?;
        worker_snapshot_envelope_artifact(snapshot)
    }

    #[wasm_bindgen(js_name = branchSnapshotArtifact)]
    pub fn branch_snapshot_artifact(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        let snapshot = self.branch_snapshot_for_test(branch_id)?;
        worker_snapshot_artifact(snapshot)
    }

    #[wasm_bindgen(js_name = branchSnapshotEnvelopeWire)]
    pub fn branch_snapshot_envelope_wire(&self, branch_id: u64) -> Result<String, JsValue> {
        store_snapshot_envelope(self.branch_snapshot_envelope_for_test(branch_id)?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = branchSnapshotEnvelopePortableWire)]
    pub fn branch_snapshot_envelope_portable_wire(
        &self,
        branch_id: u64,
    ) -> Result<String, JsValue> {
        to_json_wire(&self.branch_snapshot_envelope_for_test(branch_id)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreBranchSnapshotArtifact)]
    pub fn restore_branch_snapshot_artifact(
        &self,
        branch_id: u64,
        snapshot: JsValue,
    ) -> Result<JsValue, JsValue> {
        let snapshot: RuntimeSnapshot = from_js(snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_branch_snapshot_for_test(branch_id, snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreBranchSnapshotWire)]
    pub fn restore_branch_snapshot_wire(
        &self,
        branch_id: u64,
        snapshot: String,
    ) -> Result<JsValue, JsValue> {
        let snapshot = load_snapshot(&snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_branch_snapshot_for_test(branch_id, snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreBranchSnapshotPortableWire)]
    pub fn restore_branch_snapshot_portable_wire(
        &self,
        branch_id: u64,
        snapshot: String,
    ) -> Result<JsValue, JsValue> {
        let snapshot: RuntimeSnapshot = from_json_wire(&snapshot).map_err(JsValue::from)?;
        let envelope = self.restore_branch_snapshot_for_test(branch_id, snapshot)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = restoreBranchSnapshotById)]
    pub fn restore_branch_snapshot_by_id(
        &self,
        branch_id: u64,
        snapshot_id: u64,
    ) -> Result<JsValue, JsValue> {
        let envelope = self.restore_branch_snapshot_by_id_for_test(branch_id, snapshot_id)?;
        to_js(&envelope).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = branchStateProof)]
    pub fn branch_state_proof(&self, branch_id: u64) -> Result<JsValue, JsValue> {
        to_js(&self.branch_state_proof_for_test(branch_id)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = planMergeBranches)]
    pub fn plan_merge_branches(
        &self,
        source_branch_id: u64,
        target_branch_id: u64,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.plan_merge_branches_for_test(source_branch_id, target_branch_id)?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = planMergeBranchesWithProof)]
    pub fn plan_merge_branches_with_proof(
        &self,
        source_branch_id: u64,
        target_branch_id: u64,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.plan_merge_branches_with_proof_for_test(source_branch_id, target_branch_id)?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = mergeBranches)]
    pub fn merge_branches(
        &self,
        source_branch_id: u64,
        target_branch_id: u64,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.merge_branches_for_test(source_branch_id, target_branch_id)?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = mergeBranchesWithProof)]
    pub fn merge_branches_with_proof(
        &self,
        source_branch_id: u64,
        target_branch_id: u64,
    ) -> Result<JsValue, JsValue> {
        to_js(&self.merge_branches_with_proof_for_test(source_branch_id, target_branch_id)?)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = planMergePolicyPreview)]
    pub fn plan_merge_policy_preview(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: MergePolicyPreviewRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.plan_merge_policy_preview_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = planMergePolicyPreviewWithProof)]
    pub fn plan_merge_policy_preview_with_proof(
        &self,
        request: JsValue,
    ) -> Result<JsValue, JsValue> {
        let request: MergePolicyPreviewRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.plan_merge_policy_preview_with_proof_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = mergeBranchesPolicyPreview)]
    pub fn merge_branches_policy_preview(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: MergePolicyPreviewRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.merge_branches_policy_preview_for_test(request)?).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = mergeBranchesPolicyPreviewWithProof)]
    pub fn merge_branches_policy_preview_with_proof(
        &self,
        request: JsValue,
    ) -> Result<JsValue, JsValue> {
        let request: MergePolicyPreviewRequest = from_js(request).map_err(JsValue::from)?;
        to_js(&self.merge_branches_policy_preview_with_proof_for_test(request)?)
            .map_err(JsValue::from)
    }
}

pub(super) fn worker_snapshot_envelope_artifact(
    snapshot: RuntimeSnapshotEnvelope,
) -> Result<JsValue, JsValue> {
    let artifact = Object::new();
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshotEnvelope"),
        &to_js_structured(&snapshot).map_err(JsValue::from)?,
    )?;
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshotEnvelopeRestoreToken"),
        &JsValue::from_str(&store_snapshot_envelope(snapshot.clone()).map_err(JsValue::from)?),
    )?;
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshotEnvelopePortableWire"),
        &JsValue::from_str(&to_json_wire(&snapshot).map_err(JsValue::from)?),
    )?;
    Ok(artifact.into())
}

pub(super) fn worker_snapshot_artifact(snapshot: RuntimeSnapshot) -> Result<JsValue, JsValue> {
    let artifact = Object::new();
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshot"),
        &to_js_structured(&snapshot).map_err(JsValue::from)?,
    )?;
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshotRestoreToken"),
        &JsValue::from_str(&store_snapshot(snapshot.clone()).map_err(JsValue::from)?),
    )?;
    Reflect::set(
        &artifact,
        &JsValue::from_str("snapshotPortableWire"),
        &JsValue::from_str(&to_json_wire(&snapshot).map_err(JsValue::from)?),
    )?;
    Ok(artifact.into())
}
