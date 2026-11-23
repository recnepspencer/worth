use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use super::{RelationalForkDenial, ValidatedForkSource};
use crate::branch::{
    RelationalBranchReferenceCell, RelationalBranchReferenceObservation, RelationalBranchRoot,
    RelationalBranchVersion, RelationalForkPort, RelationalForkTargetReservation,
};
use crate::history::data::{BranchCreateError, CommitId};

/// A target cell built from the validated source, not yet installed.
pub(in crate::branch) struct PreparedForkTarget {
    pub(in crate::branch) target_cell: RelationalBranchReferenceCell,
    pub(in crate::branch) source_root: Arc<RelationalBranchRoot>,
    pub(in crate::branch) target_observation: RelationalBranchReferenceObservation,
    pub(in crate::branch) fork_provenance: RelationalBranchReferenceObservation,
    pub(in crate::branch) target_truth_version: RelationalBranchVersion,
    pub(in crate::branch) shared_commit_id: Option<CommitId>,
    pub(in crate::branch) source_head_version: Option<crate::identity::data::VersionId>,
    pub(in crate::branch) reservation: RelationalForkTargetReservation,
    pub(in crate::branch) indexes: Option<crate::indexes::PreparedForkIndexSelection>,
    pub(in crate::branch) managed_source: Option<crate::mvcc::PreparedManagedIndexSourceFork>,
}

impl RelationalForkPort {
    /// Build the target cell for the reserved destination. Nothing is
    /// installed or acquired.
    pub(super) fn prepare_fork_target(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: RelationalForkTargetReservation,
        source: &ValidatedForkSource,
        companion: Option<(u64, &dyn crate::mvcc::RelationalPublicationCompanion)>,
        lookup_cleanup: &mut Option<Box<dyn std::any::Any + Send + Sync>>,
    ) -> Result<PreparedForkTarget, RelationalForkDenial> {
        let (target_root, managed_source) = if let Some((generation, participant)) = companion {
            let (root, prepared) = crate::mvcc::PreparedManagedIndexSourceFork::prepare(
                &source.source_root,
                source.descriptor.source_branch(),
                reservation.branch_id(),
                self.runtime_instance_id,
                generation,
                participant,
                execution,
                self.owner.root_identity_issuer(),
                lookup_cleanup,
            )?;
            (root, Some(prepared))
        } else {
            (Arc::clone(&source.source_root), None)
        };
        let source_root = Arc::clone(&source.source_root);
        let target_cell = RelationalBranchReferenceCell::from_source_with_root(
            self.runtime_instance_id,
            reservation.branch_id().clone(),
            source.descriptor.source_branch().clone(),
            &source.source_observation,
            target_root,
        )
        .map_err(|_| RelationalForkDenial::InvalidTarget(BranchCreateError::invalid_target()))?;
        let target_observation = target_cell.observation().clone();
        let fork_provenance = target_cell
            .fork_provenance()
            .expect(/* Proof: Only successful fork preparation constructs this private target; it installs the selected source root and immutable fork provenance before returning it. */ "forked branch must retain exact source provenance");
        let target_truth_version = target_cell.truth_version();
        let shared_commit_id = match target_observation.target() {
            FoundationalBranchTarget::Empty => None,
            FoundationalBranchTarget::Basis(target) => Some(CommitId(target.selected_commit_id())),
        };
        let source_head_version = match shared_commit_id {
            Some(commit_id) => Some(
                source_root
                    .commit_receipt()
                    .filter(|receipt| receipt.commit_id == commit_id)
                    .map(|receipt| receipt.version_id)
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
            managed_source,
            indexes: None,
        })
    }
}
