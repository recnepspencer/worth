use serde::{Deserialize, Serialize};

use super::RecoveryCoverage;
use worth_relational_contracts::internal_port::history::data::commit_receipt::RelationalCommitReceipt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryOutcome {
    pub recovered_commits: usize,
    pub latest_commit: Option<RelationalCommitReceipt>,
    pub restored_branches: usize,
    pub coverage: RecoveryCoverage,
    #[serde(default)]
    pub checkpoint_restore_work: Option<super::CheckpointRestoreWork>,
}
