use worth_foundational::FoundationalBranchTarget;

use super::{
    BranchId, RelationalBranchCellDenial, RelationalBranchIdentity,
    RelationalBranchObservationConstructionDenial, RelationalBranchReferenceCell,
    RelationalBranchReferenceObservation, RelationalBranchVersion,
};

/// Exact durable image of one owner branch cell. This is intentionally a
/// checkpoint DTO rather than the live cell: restoring it must validate the
/// runtime-affine identity and never synthesize currentness from a legacy
/// branch-head projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RelationalBranchCellCheckpoint {
    pub(crate) runtime_instance_id: u64,
    pub(crate) branch_id: BranchId,
    pub(crate) observation: RelationalBranchReferenceObservation,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) fork_provenance: Option<RelationalBranchReferenceObservation>,
    pub(crate) fork_source_branch_id: Option<BranchId>,
}

impl RelationalBranchReferenceCell {
    /// Replace the descriptive target prepared during preflight with the
    /// storage owner's content-backed target. Generation/truth progression
    /// was already checked and advanced by `advance_truth`.
    pub(crate) fn replace_truth_target(
        &self,
        target: FoundationalBranchTarget<super::RelationalBranchTarget>,
    ) {
        if let FoundationalBranchTarget::Basis(target) = &target {
            assert_eq!(
                target.runtime_instance_id(),
                self.identity.runtime_instance_id(),
                "owner-produced content target remains runtime-affine"
            );
        }
        let mut state = self.state();
        state.observation = super::RelationalBranchReferenceObservation::new(
            state.observation.branch_id().clone(),
            target,
            state.observation.generation(),
        );
    }

    pub(crate) fn from_checkpoint_with_root(
        expected_runtime_instance_id: u64,
        checkpoint: RelationalBranchCellCheckpoint,
        root: Option<std::sync::Arc<super::super::RelationalBranchRoot>>,
    ) -> Result<Self, RelationalBranchCellDenial> {
        if checkpoint.observation.branch_id().as_str()
            != format!(
                "relational/{}/{}",
                checkpoint.runtime_instance_id, checkpoint.branch_id.0
            )
        {
            return Err(RelationalBranchCellDenial::CheckpointObservationMismatch);
        }
        if let FoundationalBranchTarget::Basis(target) = checkpoint.observation.target()
            && target.runtime_instance_id() != checkpoint.runtime_instance_id
        {
            return Err(RelationalBranchCellDenial::CheckpointObservationMismatch);
        }
        match (
            checkpoint.fork_provenance.as_ref(),
            checkpoint.fork_source_branch_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(source), Some(source_branch_id)) => {
                let expected_branch_id = format!(
                    "relational/{}/{}",
                    checkpoint.runtime_instance_id, source_branch_id.0
                );
                if source.branch_id().as_str() != expected_branch_id {
                    return Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch);
                }
                if let FoundationalBranchTarget::Basis(target) = source.target()
                    && target.runtime_instance_id() != checkpoint.runtime_instance_id
                {
                    return Err(RelationalBranchCellDenial::CheckpointObservationMismatch);
                }
            }
            _ => return Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch),
        }
        let cell = Self {
            identity: RelationalBranchIdentity::new(
                checkpoint.runtime_instance_id,
                checkpoint.branch_id.clone(),
            ),
            state: std::sync::Arc::new(std::sync::Mutex::new(
                super::RelationalBranchReferenceMutableState {
                    observation: checkpoint.observation,
                    truth_version: checkpoint.truth_version,
                    lifecycle: super::super::RelationalBranchLifecyclePosture::Live,
                    fork_provenance: checkpoint.fork_provenance,
                    fork_source_branch_id: checkpoint.fork_source_branch_id,
                    root,
                },
            )),
            basis_registry: crate::branch::RelationalBranchBasisRegistry::default(),
            coordination: crate::branch::coordination::RelationalBranchCoordinationCell::fresh(
                checkpoint.runtime_instance_id,
                &checkpoint.branch_id,
            ),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: crate::branch::RelationalBranchSharingCostCell::default(),
        };
        cell.rebind_runtime(expected_runtime_instance_id)
            .map_err(|denial| match denial {
                RelationalBranchObservationConstructionDenial::ForkProvenanceMismatch => {
                    RelationalBranchCellDenial::CheckpointForkProvenanceMismatch
                }
                _ => RelationalBranchCellDenial::CheckpointObservationMismatch,
            })
    }
}
