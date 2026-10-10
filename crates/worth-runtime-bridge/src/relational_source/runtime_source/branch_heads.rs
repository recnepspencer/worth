use crate::facade::{
    BridgeCommittedPatchEnvelope, RelationalBridgeSourceError, TruthBranchHeadSource,
    TruthBranchIdentity,
};

use super::RelationalBridgeSelectedCommitObservation;
use super::RuntimeBridgeRelationalSource;

impl RuntimeBridgeRelationalSource {
    /// Select the branch's bound head commit exactly, at constant cost.
    pub(in crate::relational_source) fn select_branch_head(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        let (commit_id, snapshot_identity) = self.branch_head_bindings.resolve(branch_identity)?;
        let observation = self.observation_bindings.resolve(&snapshot_identity)?;
        self.select_exact_commit_for_observation(commit_id, observation)
    }
}

impl TruthBranchHeadSource for RuntimeBridgeRelationalSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        execution
            .consult()
            .map_err(|denial| RelationalBridgeSourceError::execution_denied(denial.into()))?;

        let selected_commit = self.select_branch_head(branch_identity)?;

        super::publication_result::publication_envelope(
            self.publish_commit_for_selected_observation(selected_commit, execution)?,
        )
    }
}
