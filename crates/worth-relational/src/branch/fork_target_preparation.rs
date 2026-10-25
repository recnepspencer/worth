use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use super::{RelationalForkDenial, ValidatedForkSource};
use crate::branch::{
    RelationalBranchReferenceCell, RelationalBranchReferenceObservation, RelationalBranchRoot,
    RelationalBranchVersion, RelationalForkPort, RelationalForkTargetReservation,
};
use crate::history::data::{BranchCreateError, CommitId};

/// A target cell built from the validated source, not yet installed.
pub(super) struct PreparedForkTarget {
    pub(super) target_cell: RelationalBranchReferenceCell,
    pub(super) source_root: Arc<RelationalBranchRoot>,
    pub(super) target_observation: RelationalBranchReferenceObservation,
    pub(super) fork_provenance: RelationalBranchReferenceObservation,
    pub(super) target_truth_version: RelationalBranchVersion,
    pub(super) shared_commit_id: Option<CommitId>,
    pub(super) source_head_version: Option<crate::identity::data::VersionId>,
    pub(super) reservation: RelationalForkTargetReservation,
}

impl RelationalForkPort {
    /// Build the target cell for the reserved destination. Nothing is
    /// installed or acquired.
    pub(super) fn prepare_fork_target(
        &self,
        reservation: RelationalForkTargetReservation,
        source: &ValidatedForkSource,
    ) -> Result<PreparedForkTarget, RelationalForkDenial> {
        let source_root = Arc::clone(&source.source_root);
        let target_cell = RelationalBranchReferenceCell::from_source_with_root(
            self.runtime_instance_id,
            reservation.branch_id().clone(),
            source.descriptor.source_branch().clone(),
            &source.source_observation,
            source_root.clone(),
        )
        .map_err(|_| RelationalForkDenial::InvalidTarget(BranchCreateError::invalid_target()))?;
        let target_observation = target_cell.observation().clone();
        let fork_provenance = target_cell
            .fork_provenance()
            .expect("forked branch must retain exact source provenance");
        let target_truth_version = target_cell.truth_version();
        let shared_commit_id = match target_observation.target() {
            FoundationalBranchTarget::Empty => None,
            FoundationalBranchTarget::Basis(target) => Some(CommitId(target.selected_commit_id())),
        };
        let source_head_version = match shared_commit_id {
            Some(commit_id) => Some(
                source_root
                    .canonical_envelope()
                    .filter(|envelope| envelope.commit.commit_id == commit_id)
                    .map(|envelope| envelope.commit.version_id)
                    .ok_or(RelationalForkDenial::MissingArtifact)?,
            ),
            None => None,
        };
        Ok(PreparedForkTarget {
            target_cell,
            source_root,
            target_observation,
            fork_provenance,
            target_truth_version,
            shared_commit_id,
            source_head_version,
            reservation,
        })
    }
}
