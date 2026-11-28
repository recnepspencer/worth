use serde::{Deserialize, Serialize};
use worth_signal::facade::history::RuntimeBranch;

use crate::recipe::model::TransactionOp;
use crate::runtime::summaries::RunSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerBranchBasisReceipt {
    pub branch_id: u64,
    pub branch_name: String,
    pub snapshot_id: Option<u64>,
    pub native_head_generation: u64,
    pub native_head_digest: String,
    pub authored_graph_generation: u64,
    pub authored_state_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerForkBranchRequest {
    pub name: String,
    pub parent_branch_id: u64,
    pub expected_parent_basis: WorkerBranchBasisReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerForkBranchReceipt {
    pub branch: RuntimeBranch,
    pub parent_basis: WorkerBranchBasisReceipt,
    pub created_basis: WorkerBranchBasisReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerApplyTransactionToBranchRequest {
    pub branch_id: u64,
    pub expected_basis: WorkerBranchBasisReceipt,
    pub transaction_ops: Vec<TransactionOp>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerApplyTransactionToBranchReceipt {
    pub before_basis: WorkerBranchBasisReceipt,
    pub after_basis: WorkerBranchBasisReceipt,
    pub active_branch_id_before: u64,
    pub active_branch_id_after: u64,
    pub run_summary: RunSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRetireBranchRequest {
    pub branch_id: u64,
    pub expected_basis: WorkerBranchBasisReceipt,
    pub reason: WorkerBranchRetirementReason,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkerBranchRetirementReason {
    Rejected,
    Superseded,
    DependencyCancellation,
    ProjectionRebuild,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRetireBranchReceipt {
    pub retired_branch_id: u64,
    pub parent_branch_id: u64,
    pub terminal_basis: WorkerBranchBasisReceipt,
    pub closeout_digest: String,
    pub reclaimed_branch_state_count: u32,
    pub reclaimed_snapshot_state_count: u32,
    pub reclaimed_runtime_meta_count: u32,
    pub retained_proof_record_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRetireBranchesRequest {
    pub retirements: Vec<WorkerRetireBranchRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRetireBranchesReceipt {
    pub retirements: Vec<WorkerRetireBranchReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCloseoutEffectBranchRequest {
    pub canonical_transaction: WorkerApplyTransactionToBranchRequest,
    pub effect_retirement: WorkerRetireBranchRequest,
    pub dependency_basis_retirement: Option<WorkerRetireBranchRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCloseoutEffectBranchReceipt {
    pub canonical_transaction: WorkerApplyTransactionToBranchReceipt,
    pub effect_retirement: WorkerRetireBranchReceipt,
    pub dependency_basis_retirement: Option<WorkerRetireBranchReceipt>,
}
