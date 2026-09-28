use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, RelationalBridgeSourceError, TruthBranchHeadSource,
    TruthBranchIdentity,
};

use super::selected_commit_resolution::SourceCommitSelection;
use super::RuntimeBridgeRelationalSource;

impl RuntimeBridgeRelationalSource {
    /// Select the branch's bound head commit exactly, at constant cost.
    pub(in crate::presentation::bridge) fn select_branch_head(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<SourceCommitSelection, RelationalBridgeSourceError> {
        let (commit_id, snapshot_identity) = self.branch_head_bindings.resolve(branch_identity)?;
        let observation = self.observation_bindings.resolve(&snapshot_identity)?;
        Ok(self.select_exact_commit_for_observation(commit_id, observation))
    }
}

impl TruthBranchHeadSource for RuntimeBridgeRelationalSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        let selected_commit = self.select_branch_head(branch_identity)?.into_result()?;

        match self.publish_commit_for_selected_observation(selected_commit) {
            worth_proof::TransitionOutcome::Success(publication) => {
                Ok(publication.into_bridge_envelope())
            }
            worth_proof::TransitionOutcome::Denied(denial) => {
                Err(RelationalBridgeSourceError::new(format!(
                    "relational branch-head patch could not be admitted by Bridge: {denial}"
                )))
            }
            worth_proof::TransitionOutcome::Deferred(_) => Err(RelationalBridgeSourceError::new(
                "relational branch-head publication deferred",
            )),
            worth_proof::TransitionOutcome::Stale(_) => Err(RelationalBridgeSourceError::new(
                "relational branch-head authority is stale",
            )),
            worth_proof::TransitionOutcome::RebindRequired(_) => Err(
                RelationalBridgeSourceError::new("relational branch-head requires graph rebind"),
            ),
            worth_proof::TransitionOutcome::Failed(_) => Err(RelationalBridgeSourceError::new(
                "relational branch-head lowering failed",
            )),
        }
    }
}
