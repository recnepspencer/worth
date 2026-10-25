use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use super::RelationalForkDenial;
use crate::branch::fork_source_basis::RelationalForkSourceDescriptor;
use crate::branch::{
    RelationalBranchLifecyclePosture, RelationalBranchReferenceObservation, RelationalBranchRoot,
    RelationalBranchTarget, RelationalBranchTransition, RelationalBranchVersion,
    RelationalForkPort, CANONICAL_RUNTIME_INSTANCE_ID,
};
use crate::history::data::BranchId;

/// A source proven live and unchanged since its fork token was issued.
pub(super) struct ValidatedForkSource {
    pub(super) descriptor: RelationalForkSourceDescriptor,
    pub(super) source_observation: RelationalBranchReferenceObservation,
    pub(super) source_truth_version: RelationalBranchVersion,
    pub(super) source_root: Arc<RelationalBranchRoot>,
    /// The source's selected head in the canonical runtime frame.
    source_head: RelationalBranchTarget,
}

impl ValidatedForkSource {
    /// The durable transition that forks `target` from this source.
    pub(super) fn transition(&self, target: BranchId) -> RelationalBranchTransition {
        RelationalBranchTransition::Fork {
            target,
            source: self.descriptor.source_branch().clone(),
            source_generation: self.source_observation.generation(),
            source_truth_version: self.source_truth_version,
            source_head: self.source_head.clone(),
        }
    }
}

impl RelationalForkPort {
    /// Require the source the token describes to be live and unchanged.
    pub(super) fn validate_fork_source(
        &self,
        descriptor: RelationalForkSourceDescriptor,
    ) -> Result<ValidatedForkSource, RelationalForkDenial> {
        if descriptor.runtime_instance_id() != self.runtime_instance_id {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        let (source_observation, source_truth_version, source_root) =
            self.observe_live_source(descriptor.source_branch())?;
        self.owner.record_lookup();
        if descriptor.truth_version() != source_truth_version
            || descriptor
                .observation()
                .compare(&source_observation)
                .is_err()
        {
            return Err(RelationalForkDenial::StaleSource);
        }
        let source_head = match source_observation.target() {
            FoundationalBranchTarget::Empty => return Err(RelationalForkDenial::EmptySource),
            FoundationalBranchTarget::Basis(target) => {
                target.rebind_runtime_instance_id(CANONICAL_RUNTIME_INSTANCE_ID)
            }
        };
        Ok(ValidatedForkSource {
            descriptor,
            source_observation,
            source_truth_version,
            source_root: source_root.ok_or(RelationalForkDenial::MissingArtifact)?,
            source_head,
        })
    }

    /// Inside the patch-position reservation, where no commit or transition
    /// can move the source: require it still to be exactly the validated
    /// source, so the installed target never shares a superseded head.
    ///
    /// The source guard was acquired before the reservation. Rechecking never
    /// locks a branch while holding the global reservation.
    pub(super) fn recheck_fork_source(
        &self,
        validated: &ValidatedForkSource,
        state: &crate::branch::reference_publication_cell::RelationalBranchPublicationStateGuard<
            '_,
        >,
    ) -> Result<(), RelationalForkDenial> {
        match state.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived)
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting)
            }
        }
        if state.truth_version() != validated.source_truth_version
            || validated
                .source_observation
                .compare(state.observation())
                .is_err()
        {
            return Err(RelationalForkDenial::StaleSource);
        }
        Ok(())
    }

    fn observe_live_source(
        &self,
        source_branch: &BranchId,
    ) -> Result<
        (
            RelationalBranchReferenceObservation,
            RelationalBranchVersion,
            Option<Arc<RelationalBranchRoot>>,
        ),
        RelationalForkDenial,
    > {
        let source_cell = self
            .owner
            .branch_cell(source_branch)
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        let snapshot = source_cell.atomic_snapshot();
        match snapshot.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived);
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting);
            }
        }
        Ok((
            snapshot.observation(),
            snapshot.truth_version(),
            snapshot.root(),
        ))
    }
}
