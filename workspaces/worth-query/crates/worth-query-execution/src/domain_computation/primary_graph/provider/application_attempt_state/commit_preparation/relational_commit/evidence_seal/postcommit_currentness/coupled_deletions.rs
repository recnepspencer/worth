//! The entities one coupled commit deleted, collected once for all its writers.
use super::*;
use std::sync::OnceLock;
use worth_relational::facade::publication::RecordStructuralChange;
use worth_relational::facade::transactions::{CommitResult, RecordRef};

/// One per coupled commit, shared by the root's and every member's session.
/// The first writer's rebase walks the commit's patches; the others read the
/// same set, so the walk happens once per commit and not once per writer.
#[derive(Clone, Default)]
pub struct CoupledCommitDeletions(Arc<OnceLock<Arc<BTreeSet<EntityId>>>>);

impl CoupledCommitDeletions {
    pub(super) fn collect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: &CommitResult,
    ) -> Result<Arc<BTreeSet<EntityId>>, CompanionPreflightStop> {
        if let Some(deleted) = self.0.get() {
            return Ok(Arc::clone(deleted));
        }
        let mut deleted = BTreeSet::new();
        for patch in commit.patch() {
            execution
                .consult()
                .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
            walk_observation::record();
            if patch.structural_change == RecordStructuralChange::Deleted
                && let RecordRef::Entity(entity) = patch.target
            {
                deleted.insert(entity);
            }
        }
        Ok(Arc::clone(self.0.get_or_init(|| Arc::new(deleted))))
    }
}

mod walk_observation;
