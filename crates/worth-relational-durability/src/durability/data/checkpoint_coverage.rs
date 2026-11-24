use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port as contracts;

use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::identity::data::primitives::VersionId;

/// The last commit, version and branch transition a checkpoint image
/// includes. Recovery replays only the commits and transitions after them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointCoverage {
    pub up_to_commit: Option<RelationalCommitReceipt>,
    pub up_to_version: Option<VersionId>,
    pub up_to_branch_transition:
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
}
