//! Observe the actual live source and issue its fork-only token.
use super::*;
impl RelationalForkPort {
    /// Observe an exact live source and issue the owner-sealed fork-only
    /// token. Empty branches do not produce a fork source.
    pub fn observe_fork_source(
        &self,
        source_branch: &BranchId,
    ) -> Result<
        (
            RelationalForkSourceDescriptor,
            AdmittedRelationalForkSourceBasis,
        ),
        RelationalForkDenial,
    > {
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        let source_cell = self
            .owner
            .branch_cell(source_branch)
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        let source_snapshot = source_cell.atomic_snapshot();
        match source_snapshot.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived);
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting);
            }
        }
        let observation = source_snapshot.observation();
        let truth_version = source_snapshot.truth_version();
        self.owner.record_lookup();
        if observation.target().is_empty() {
            return Err(RelationalForkDenial::EmptySource);
        }
        let descriptor = RelationalForkSourceDescriptor::new(
            self.runtime_instance_id,
            observation,
            source_branch.clone(),
            truth_version,
        );
        let token = admit_relational_fork_source(descriptor.clone());
        Ok((descriptor, token))
    }
}
