use serde::{Deserialize, Serialize};

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyFreezeMode {
    FreezeAtCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompiledArtifactAuthorityStatus {
    Authoritative,
    StaleVersion,
    MissingSourceCommit,
    CompiledLaneDisabled,
    /// The caller's request or the source envelope demand was refused.
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledArtifactError {
    pub authority_status: CompiledArtifactAuthorityStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledExecutionArtifact {
    pub artifact_id: u64,
    pub source_commit_id: CommitId,
    pub source_version_id: VersionId,
    pub source_branch_id: BranchId,
    pub partition_ids: Vec<PartitionId>,
    pub topology_freeze_mode: TopologyFreezeMode,
    pub compiled_record_count: usize,
}
