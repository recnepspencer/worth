use serde::{Deserialize, Serialize};
use worth_foundational::FoundationalBranchReferenceGeneration;

use crate::branch::{RelationalBranchTarget, RelationalBranchVersion};
use crate::history::data::{BranchId, CommitId};

/// One branch lifecycle transition as durable history records it.
///
/// It is runtime-neutral (D11): branches are named, never identified by a
/// process-local runtime, and a fork's source head is a branch target in the
/// canonical runtime frame. Two runtimes performing the same transition on
/// the same durable history record the same value.
///
/// Every variant names the exact prior state it was performed against, so
/// recovery can refuse a transition that does not apply to the branch it
/// replays onto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RelationalBranchTransition {
    /// `target` came into existence as a fork of `source` at `source_head`.
    Fork {
        target: BranchId,
        source: BranchId,
        source_generation: FoundationalBranchReferenceGeneration,
        source_truth_version: RelationalBranchVersion,
        source_head: RelationalBranchTarget,
    },
    /// `branch` stopped accepting commits; its generation advanced.
    Archive {
        branch: BranchId,
        prior_generation: FoundationalBranchReferenceGeneration,
        prior_truth_version: RelationalBranchVersion,
    },
    /// `branch` was marked for deletion while operations still held it; its
    /// name is retired.
    DeletionRequested {
        branch: BranchId,
        prior_generation: FoundationalBranchReferenceGeneration,
        prior_truth_version: RelationalBranchVersion,
    },
    /// `branch` was removed and its head, `head_commit`, retired; its name is
    /// retired.
    Delete {
        branch: BranchId,
        prior_generation: FoundationalBranchReferenceGeneration,
        prior_truth_version: RelationalBranchVersion,
        head_commit: Option<CommitId>,
    },
}

impl RelationalBranchTransition {
    /// The branch this transition changes: a fork's target, otherwise the
    /// archived or deleted branch.
    pub fn branch(&self) -> &BranchId {
        match self {
            Self::Fork { target, .. } => target,
            Self::Archive { branch, .. }
            | Self::DeletionRequested { branch, .. }
            | Self::Delete { branch, .. } => branch,
        }
    }
}
