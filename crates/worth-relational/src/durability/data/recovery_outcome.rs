use serde::{Deserialize, Serialize};

use super::{
    DurableCheckpointId, DurableSegmentId, RecoveryCoverage, RecoveryCursor,
    RecoveryIntegrityReport,
};
use crate::history::data::RelationalCommitReceipt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryOutcome {
    pub recovered_commits: usize,
    pub latest_commit: Option<RelationalCommitReceipt>,
    pub restored_branches: usize,
    pub cursor: RecoveryCursor,
    pub coverage: RecoveryCoverage,
    pub integrity_report: RecoveryIntegrityReport,
    #[serde(default)]
    pub checkpoint_restore_work: Option<super::CheckpointRestoreWork>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionPlan {
    pub checkpoint_id: DurableCheckpointId,
    pub removable_segments: Vec<DurableSegmentId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionOutcome {
    pub removed_segments: Vec<DurableSegmentId>,
    pub retained_segments: Vec<DurableSegmentId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionPolicy {
    pub remove_fully_covered_segments: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentRetentionClass {
    CoveredByCheckpoint,
    RequiredForRecovery,
}
