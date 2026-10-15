use std::sync::{Arc, Mutex};
use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

use super::identity::RelationalBranchIdentity;
use super::reference_state::RelationalBranchReferenceState;
use super::target::RelationalBranchTarget;
use super::RelationalBranchVersion;
use crate::history::data::BranchId;

#[path = "reference_observation.rs"]
mod observation;
pub use observation::{
    relational_branch_observation, RelationalBranchComparisonBasis, RelationalBranchForkBasis,
    RelationalBranchObservationConstructionDenial, RelationalBranchReferenceObservation,
};

/// Mutable owner cell for one branch reference. The exact Foundational observation
/// owns target/generation; owner-local truth movement stays in the version counter.
#[derive(Debug)]
pub(crate) struct RelationalBranchReferenceCell {
    pub(super) identity: RelationalBranchIdentity,
    pub(super) state: Arc<Mutex<RelationalBranchReferenceMutableState>>,
    pub(super) basis_registry: super::RelationalBranchBasisRegistry,
    pub(super) coordination: Arc<super::coordination::RelationalBranchCoordinationCell>,
    pub(super) head_retention: Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
    pub(super) sharing_costs: super::RelationalBranchSharingCostCell,
}

impl Clone for RelationalBranchReferenceCell {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            state: Arc::clone(&self.state),
            basis_registry: self.basis_registry.clone(),
            coordination: Arc::clone(&self.coordination),
            head_retention: Arc::clone(&self.head_retention),
            sharing_costs: self.sharing_costs.clone(),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchReferenceMutableState {
    pub(super) observation: RelationalBranchReferenceObservation,
    pub(super) truth_version: RelationalBranchVersion,
    pub(super) lifecycle: super::RelationalBranchLifecyclePosture,
    fork_provenance: Option<RelationalBranchReferenceObservation>,
    fork_source_branch_id: Option<BranchId>,
    pub(super) root: Option<Arc<super::RelationalBranchRoot>>,
}

impl RelationalBranchReferenceMutableState {
    pub(super) fn currently_selects_root(
        &self,
        expected: &Arc<super::RelationalBranchRoot>,
    ) -> bool {
        self.root
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, expected))
    }

    pub(crate) const fn lifecycle_posture(&self) -> super::RelationalBranchLifecyclePosture {
        self.lifecycle
    }
}

impl RelationalBranchReferenceCell {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            state: Arc::new(Mutex::new(self.state_snapshot())),
            basis_registry: super::RelationalBranchBasisRegistry::default(),
            coordination: super::coordination::RelationalBranchCoordinationCell::fresh(
                self.identity.runtime_instance_id(),
                self.identity.branch_id(),
            ),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: self.sharing_costs.detached_owner_snapshot(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBranchCellDenial {
    GenerationOverflow,
    TruthVersionOverflow,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwnerUnavailable,
    RetentionRootSetTooLarge,
    RuntimeInstanceMismatch,
    BranchIdentityMismatch,
    CheckpointRuntimeMismatch,
    CheckpointObservationMismatch,
    CheckpointForkProvenanceMismatch,
    CheckpointRootReadmissionRequired,
}

impl RelationalBranchReferenceCell {
    pub(crate) fn clone_for_head_replacement(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            state: Arc::new(Mutex::new(self.state_snapshot())),
            basis_registry: self.basis_registry.clone(),
            coordination: Arc::clone(&self.coordination),
            head_retention: Arc::clone(&self.head_retention),
            sharing_costs: self.sharing_costs.clone(),
        }
    }

    /// Detach one exact image of every mutable branch-reference axis.
    ///
    /// Public observations and fork admission must derive reference, truth,
    /// and root evidence from this single lock acquisition.
    pub(crate) fn atomic_snapshot(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            state: Arc::new(Mutex::new(self.state_snapshot())),
            basis_registry: self.basis_registry.clone(),
            coordination: Arc::clone(&self.coordination),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: super::RelationalBranchSharingCostCell::default(),
        }
    }

    pub(crate) fn empty(
        runtime_instance_id: u64,
        branch_id: BranchId,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        let observation = relational_branch_observation(
            runtime_instance_id,
            &branch_id.0,
            FoundationalBranchTarget::empty(),
            FoundationalBranchReferenceGeneration::initial(),
        )?;
        Ok(Self {
            identity: RelationalBranchIdentity::new(runtime_instance_id, branch_id.clone()),
            state: Arc::new(Mutex::new(RelationalBranchReferenceMutableState {
                observation,
                truth_version: RelationalBranchVersion::initial(),
                lifecycle: super::RelationalBranchLifecyclePosture::Live,
                fork_provenance: None,
                fork_source_branch_id: None,
                root: None,
            })),
            basis_registry: super::RelationalBranchBasisRegistry::default(),
            coordination: super::coordination::RelationalBranchCoordinationCell::fresh(
                runtime_instance_id,
                &branch_id,
            ),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: super::RelationalBranchSharingCostCell::default(),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_source(
        runtime_instance_id: u64,
        branch_id: BranchId,
        source_branch_id: BranchId,
        source: &RelationalBranchReferenceObservation,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        Self::from_source_with_root(
            runtime_instance_id,
            branch_id,
            source_branch_id,
            source,
            super::RelationalBranchRoot::empty(),
        )
    }

    pub(crate) fn from_source_with_root(
        runtime_instance_id: u64,
        branch_id: BranchId,
        source_branch_id: BranchId,
        source: &RelationalBranchReferenceObservation,
        source_root: Arc<super::RelationalBranchRoot>,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        let target = match source.target() {
            FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
            FoundationalBranchTarget::Basis(target) => FoundationalBranchTarget::basis(
                target.rebind_runtime_instance_id(runtime_instance_id),
            ),
        };
        let observation = relational_branch_observation(
            runtime_instance_id,
            &branch_id.0,
            target,
            FoundationalBranchReferenceGeneration::initial(),
        )?;
        Ok(Self {
            identity: RelationalBranchIdentity::new(runtime_instance_id, branch_id.clone()),
            state: Arc::new(Mutex::new(RelationalBranchReferenceMutableState {
                observation,
                truth_version: RelationalBranchVersion::initial(),
                lifecycle: super::RelationalBranchLifecyclePosture::Live,
                fork_provenance: Some(source.clone()),
                fork_source_branch_id: Some(source_branch_id),
                root: Some(source_root),
            })),
            basis_registry: super::RelationalBranchBasisRegistry::default(),
            coordination: super::coordination::RelationalBranchCoordinationCell::fresh(
                runtime_instance_id,
                &branch_id,
            ),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: super::RelationalBranchSharingCostCell::default(),
        })
    }

    pub(crate) fn identity(&self) -> &RelationalBranchIdentity {
        &self.identity
    }

    pub(crate) fn checkpoint(&self) -> RelationalBranchCellCheckpoint {
        let state = self.state_snapshot();
        RelationalBranchCellCheckpoint {
            runtime_instance_id: self.identity.runtime_instance_id(),
            branch_id: self.identity.branch_id().clone(),
            observation: state.observation,
            truth_version: state.truth_version,
            fork_provenance: state.fork_provenance,
            fork_source_branch_id: state.fork_source_branch_id,
        }
    }

    pub(crate) fn evidence_state(&self) -> RelationalBranchReferenceState {
        let state = self.state_snapshot();
        RelationalBranchReferenceState::new(
            self.identity.runtime_instance_id(),
            self.identity.branch_id().clone(),
            state.observation,
            state.truth_version,
            state.lifecycle,
            state.fork_provenance,
            state.fork_source_branch_id,
        )
    }

    pub(crate) fn observation(&self) -> RelationalBranchReferenceObservation {
        self.state_snapshot().observation
    }

    pub(crate) fn truth_version(&self) -> RelationalBranchVersion {
        self.state_snapshot().truth_version
    }

    pub(crate) fn lifecycle_posture(&self) -> super::RelationalBranchLifecyclePosture {
        self.state_snapshot().lifecycle
    }

    pub(crate) fn fork_provenance(&self) -> Option<RelationalBranchReferenceObservation> {
        self.state_snapshot().fork_provenance
    }

    pub(crate) fn fork_source_branch_id(&self) -> Option<BranchId> {
        self.state_snapshot().fork_source_branch_id
    }

    pub(crate) fn advance_metadata(&self) -> Result<(), RelationalBranchCellDenial> {
        self.advance_metadata_to(self.observation().target().clone())
    }

    pub(crate) fn advance_metadata_to(
        &self,
        target: FoundationalBranchTarget<RelationalBranchTarget>,
    ) -> Result<(), RelationalBranchCellDenial> {
        if let FoundationalBranchTarget::Basis(target) = &target
            && target.runtime_instance_id() != self.identity.runtime_instance_id()
        {
            return Err(RelationalBranchCellDenial::RuntimeInstanceMismatch);
        }
        let mut state = self.state();
        state.observation = RelationalBranchReferenceObservation::new(
            state.observation.branch_id().clone(),
            target,
            state
                .observation
                .generation()
                .checked_advance()
                .map_err(|_| RelationalBranchCellDenial::GenerationOverflow)?,
        );
        Ok(())
    }

    pub(crate) fn advance_truth(
        &self,
        target: FoundationalBranchTarget<RelationalBranchTarget>,
    ) -> Result<(), RelationalBranchCellDenial> {
        let mut state = self.state();
        let generation = state
            .observation
            .generation()
            .checked_advance()
            .map_err(|_| RelationalBranchCellDenial::GenerationOverflow)?;
        let truth_version = state
            .truth_version
            .checked_advance()
            .ok_or(RelationalBranchCellDenial::TruthVersionOverflow)?;
        let candidate = RelationalBranchReferenceObservation::new(
            state.observation.branch_id().clone(),
            target,
            generation,
        );
        if let FoundationalBranchTarget::Basis(target) = candidate.target()
            && target.runtime_instance_id() != self.identity.runtime_instance_id()
        {
            return Err(RelationalBranchCellDenial::RuntimeInstanceMismatch);
        }
        state.observation = candidate;
        state.truth_version = truth_version;
        Ok(())
    }

    pub(crate) fn state_snapshot(&self) -> RelationalBranchReferenceMutableState {
        self.state().clone()
    }

    pub(crate) fn replace_state(&self, next: RelationalBranchReferenceMutableState) {
        *self.state() = next;
    }

    pub(super) fn state(&self) -> std::sync::MutexGuard<'_, RelationalBranchReferenceMutableState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "reference_tests.rs"]
mod tests;

#[path = "reference_checkpoint.rs"]
mod checkpoint;
pub(crate) use checkpoint::RelationalBranchCellCheckpoint;
#[path = "reference_coordination_access.rs"]
mod coordination_access;
#[path = "reference_root_access.rs"]
mod root_access;
#[path = "reference_runtime_rebinding.rs"]
mod runtime_rebinding;
