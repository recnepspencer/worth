use super::*;
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};

pub trait CommittedPatchSource: Send + Sync + 'static {
    fn authoritative_source_profile(
        &self,
    ) -> Option<crate::input::envelope::BridgeAuthoritativeSourceProfile> {
        None
    }

    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::input::envelope::BridgeCommittedPatchEnvelope, RelationalBridgeSourceError>;
}

pub trait SnapshotReadSource: Send + Sync + 'static {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError>;
}

pub trait SnapshotReaderPool: Send + Sync + 'static {
    fn acquire(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError>;

    fn release(&self, reader: Box<dyn TruthSnapshotReader>);
}

pub trait TruthBranchHeadSource: Send + Sync + 'static {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::input::envelope::BridgeCommittedPatchEnvelope, RelationalBridgeSourceError>;
}

pub trait RelationalBridgeSource:
    CommittedPatchSource + SnapshotReadSource + TruthBranchHeadSource
{
}

impl<T> RelationalBridgeSource for T where
    T: CommittedPatchSource + SnapshotReadSource + TruthBranchHeadSource
{
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalCommittedPatchRequest {
    commit_identity: TruthCommitIdentity,
    branch_identity: Option<TruthBranchIdentity>,
    snapshot_identity: Option<TruthSnapshotIdentity>,
}

impl RelationalCommittedPatchRequest {
    pub fn new(commit_identity: TruthCommitIdentity) -> Self {
        Self {
            commit_identity,
            branch_identity: None,
            snapshot_identity: None,
        }
    }

    pub fn on_branch(
        commit_identity: TruthCommitIdentity,
        branch_identity: TruthBranchIdentity,
    ) -> Self {
        Self {
            commit_identity,
            branch_identity: Some(branch_identity),
            snapshot_identity: None,
        }
    }

    pub fn at_snapshot(
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self {
            commit_identity,
            branch_identity: None,
            snapshot_identity: Some(snapshot_identity),
        }
    }

    pub fn commit_identity(&self) -> &TruthCommitIdentity {
        &self.commit_identity
    }

    pub fn branch_identity(&self) -> Option<&TruthBranchIdentity> {
        self.branch_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> Option<&TruthSnapshotIdentity> {
        self.snapshot_identity.as_ref()
    }
}
