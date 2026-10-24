use serde::{Deserialize, Serialize};

use super::RecoveryCoverage;
use crate::history::data::RelationalCommitReceipt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryOutcome {
    pub recovered_commits: usize,
    pub latest_commit: Option<RelationalCommitReceipt>,
    pub restored_branches: usize,
    pub coverage: RecoveryCoverage,
    #[serde(default)]
    pub checkpoint_restore_work: Option<super::CheckpointRestoreWork>,
}
