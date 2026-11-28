use serde::{Deserialize, Serialize};

use crate::logic::transaction::runtime::state::canonical_digest;

pub const BRANCH_STATE_PROOF_BASIS_VERSION: &str = "worth-signal-branch-state-v3";
pub const BRANCH_STATE_PROOF_SCHEMA_VERSION: &str = "worth-signal-proof-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStateDenseGridProofBasis {
    pub family_id: String,
    pub width: u32,
    pub height: u32,
    pub key_count: usize,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStateProofBasis<TStore> {
    pub proof_schema_version: String,
    pub catalog_ids: Vec<String>,
    pub dense_grids: Vec<BranchStateDenseGridProofBasis>,
    pub store: TStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStateProofReport {
    pub proof_schema_version: String,
    pub branch_id: u64,
    pub branch_name: String,
    pub snapshot_id: Option<u64>,
    pub state_digest: String,
}

pub fn branch_state_proof_report<T: Serialize>(
    branch_id: u64,
    branch_name: impl Into<String>,
    snapshot_id: Option<u64>,
    basis_version: &str,
    state_basis: &T,
) -> BranchStateProofReport {
    BranchStateProofReport {
        proof_schema_version: format!("{BRANCH_STATE_PROOF_SCHEMA_VERSION}:{basis_version}"),
        branch_id,
        branch_name: branch_name.into(),
        snapshot_id,
        state_digest: canonical_digest(state_basis),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReplayMismatchClass {
    ProofSchemaVersionMismatch,
    MissingRegistryBundleDigest,
    RegistryBundleDigestMismatch,
    BranchStateDigestMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayParityProofReport {
    pub proof_schema_version: String,
    pub expected_branch_id: u64,
    pub expected_branch_name: String,
    pub expected_snapshot_id: Option<u64>,
    pub expected_state_digest: String,
    pub replayed_branch_id: u64,
    pub replayed_branch_name: String,
    pub replayed_snapshot_id: Option<u64>,
    pub replayed_state_digest: String,
    pub parity: bool,
    pub mismatch_classes: Vec<ReplayMismatchClass>,
}

pub fn replay_parity_proof_report(
    expected_branch_id: u64,
    expected_branch_name: impl Into<String>,
    expected_snapshot_id: Option<u64>,
    expected_state_digest: impl Into<String>,
    replayed_branch_id: u64,
    replayed_branch_name: impl Into<String>,
    replayed_snapshot_id: Option<u64>,
    replayed_state_digest: impl Into<String>,
) -> ReplayParityProofReport {
    let expected_state_digest = expected_state_digest.into();
    let replayed_state_digest = replayed_state_digest.into();
    let mismatch_classes = if expected_state_digest == replayed_state_digest {
        Vec::new()
    } else {
        vec![ReplayMismatchClass::BranchStateDigestMismatch]
    };
    ReplayParityProofReport {
        proof_schema_version: BRANCH_STATE_PROOF_SCHEMA_VERSION.to_owned(),
        expected_branch_id,
        expected_branch_name: expected_branch_name.into(),
        expected_snapshot_id,
        expected_state_digest: expected_state_digest.clone(),
        replayed_branch_id,
        replayed_branch_name: replayed_branch_name.into(),
        replayed_snapshot_id,
        replayed_state_digest: replayed_state_digest.clone(),
        parity: mismatch_classes.is_empty(),
        mismatch_classes,
    }
}
