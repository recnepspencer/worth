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

/// Temporary source custody for a capture already holding the original
/// configuration/publication epochs. No name-bearing handle is cloned.
pub(in crate::branch) struct CheckpointCellSource {
    runtime_instance_id: u64,
    state: std::sync::Arc<std::sync::Mutex<super::RelationalBranchReferenceMutableState>>,
}

impl CheckpointCellSource {
    pub(in crate::branch) fn matches(&self, cell: &RelationalBranchReferenceCell) -> bool {
        self.runtime_instance_id == cell.identity().runtime_instance_id()
            && std::sync::Arc::ptr_eq(&self.state, &cell.state)
    }

    pub(in crate::branch) fn nested_allocation_bytes(&self) -> Option<usize> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut bytes = observation_bytes(&state.observation)?;
        if let Some(provenance) = &state.fork_provenance {
            bytes = bytes.checked_add(observation_bytes(provenance)?)?;
        }
        if let Some(source) = &state.fork_source_branch_id {
            bytes = bytes.checked_add(source.0.len())?;
        }
        Some(bytes)
    }

    pub(in crate::branch) fn copy(
        &self,
        branch_id: BranchId,
    ) -> (
        RelationalBranchCellCheckpoint,
        crate::durability::data::DurableBranchLifecycleImage,
    ) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let lifecycle = crate::durability::data::DurableBranchLifecycleImage {
            branch_id: branch_id.clone(),
            posture: state.lifecycle,
        };
        (
            RelationalBranchCellCheckpoint {
                runtime_instance_id: self.runtime_instance_id,
                branch_id,
                observation: state.observation.clone(),
                truth_version: state.truth_version,
                fork_provenance: state.fork_provenance.clone(),
                fork_source_branch_id: state.fork_source_branch_id.clone(),
            },
            lifecycle,
        )
    }
}

fn observation_bytes(observation: &RelationalBranchReferenceObservation) -> Option<usize> {
    let parents = match observation.target() {
        FoundationalBranchTarget::Empty => 0,
        FoundationalBranchTarget::Basis(target) => {
            std::alloc::Layout::array::<u64>(target.parent_commit_ids().len())
                .ok()?
                .size()
        }
    };
    observation.branch_id().as_str().len().checked_add(parents)
}

impl RelationalBranchReferenceCell {
    pub(in crate::branch) fn checkpoint_source(&self) -> CheckpointCellSource {
        CheckpointCellSource {
            runtime_instance_id: self.identity().runtime_instance_id(),
            state: std::sync::Arc::clone(&self.state),
        }
    }
    /// Original Empty schema DATA and axes under one actual state borrow.
    /// This does not clone the name-bearing publication handle or retain a root.
    pub(crate) fn retained_empty_schema_data(
        &self,
    ) -> Option<(
        worth_foundational::FoundationalBranchReferenceGeneration,
        RelationalBranchVersion,
        crate::branch::RelationalBranchLifecyclePosture,
        std::sync::Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
    )> {
        let state = self.state();
        if !matches!(state.observation.target(), FoundationalBranchTarget::Empty) {
            return None;
        }
        Some((
            state.observation.generation(),
            state.truth_version,
            state.lifecycle,
            state.root.as_ref()?.retained_schema_authority(),
        ))
    }

    /// Replace the descriptive target prepared during preflight with the
    /// storage owner's content-backed target. Generation/truth progression
    /// was already checked and advanced by `advance_truth`.
    pub(crate) fn replace_truth_target(
        &self,
        target: FoundationalBranchTarget<super::RelationalBranchTarget>,
    ) -> Result<(), RelationalBranchObservationConstructionDenial> {
        if let FoundationalBranchTarget::Basis(target) = &target
            && target.runtime_instance_id() != self.identity.runtime_instance_id()
        {
            return Err(
                RelationalBranchObservationConstructionDenial::RuntimeInstanceMismatch {
                    observation_runtime_instance_id: self.identity.runtime_instance_id(),
                    target_runtime_instance_id: target.runtime_instance_id(),
                },
            );
        }
        let mut state = self.state();
        state.observation = super::RelationalBranchReferenceObservation::new(
            state.observation.branch_id().clone(),
            target,
            state.observation.generation(),
        );
        Ok(())
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
