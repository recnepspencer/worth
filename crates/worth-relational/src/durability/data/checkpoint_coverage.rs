use serde::{Deserialize, Serialize};

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::VersionId;

/// The last commit and version a checkpoint image includes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointCoverage {
    pub up_to_commit: Option<RelationalCommitReceipt>,
    pub up_to_version: Option<VersionId>,
}
