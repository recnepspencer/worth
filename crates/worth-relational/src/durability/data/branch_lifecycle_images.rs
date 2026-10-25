use serde::{Deserialize, Serialize};

use crate::branch::RelationalBranchLifecyclePosture;
use crate::history::data::BranchId;

/// The lifecycle posture a checkpoint records for one branch it holds. A
/// checkpoint records one for every branch cell, so recovery restores an
/// archive or a pending deletion instead of reviving the branch as live.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableBranchLifecycleImage {
    pub(crate) branch_id: BranchId,
    pub(crate) posture: RelationalBranchLifecyclePosture,
}
