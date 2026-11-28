use worth_execution::ExecutionRequest;
use worth_relational::facade::change_source::RelationalPinnedCommitDenial;

use crate::adapter::RelationalBridgeSourceErrorTag;
use crate::facade::{
    RelationalBridgeSourceError, TruthBranchIdentity, TruthCommitIdentity, TruthSnapshotIdentity,
};
use crate::relational_source::identities::parse_bridge_commit_identity;
use crate::relational_source::snapshot_reading::RuntimePublicationSnapshotReader;

use super::RuntimeBridgeRelationalSource;

impl RuntimeBridgeRelationalSource {
    pub(crate) fn shares_observation_bindings(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.observation_bindings, &other.observation_bindings)
    }

    pub(crate) fn pin_commit_snapshot(
        &self,
        branch: &TruthBranchIdentity,
        commit: &TruthCommitIdentity,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<RuntimePublicationSnapshotReader, RelationalBridgeSourceError> {
        crate::execution_contact::admit(execution)
            .map_err(RelationalBridgeSourceError::execution_denied)?;
        let requested = parse_bridge_commit_identity(commit)?;
        let (_, head) = self.branch_head_bindings.resolve(branch)?;
        let observation = self.observation_bindings.resolve(&head)?;
        let pinned = self
            .runtime
            .with_runtime(|runtime| {
                runtime.pin_commit_observation(execution, observation.observation(), requested)
            })
            .map_err(|denial| {
                let kind = match denial {
                    RelationalPinnedCommitDenial::Selection(selection) => {
                        RelationalBridgeSourceErrorTag::CommitSelection(selection)
                    }
                    other => RelationalBridgeSourceErrorTag::PinnedCommit(other),
                };
                RelationalBridgeSourceError::new(
                    kind,
                    format!("could not pin historical commit: {denial:?}"),
                )
            })?;
        let snapshot = TruthSnapshotIdentity::admit_bridge_owned(format!(
            "relational-pinned-commit:{}:{}",
            requested.0,
            pinned.root_identity(),
        ));
        Ok(RuntimePublicationSnapshotReader::for_pinned_commit(
            self.runtime.clone(),
            snapshot,
            pinned,
            self.partition
                .as_ref()
                .map(|partition| partition.relational),
        ))
    }
}
