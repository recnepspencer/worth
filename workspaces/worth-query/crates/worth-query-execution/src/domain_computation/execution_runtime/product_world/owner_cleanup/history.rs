use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    WorthQueryProductBranchHistoryCleanup, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductBranchOwnerCleanupRecord,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

/// What follows every history release a branch or recovery cleanup makes.
pub(crate) trait WorthQueryReleasedHistoryFollower: Send + Sync {
    /// History a walk released may have been the last to keep evidence Query
    /// inherited onto its commits; that evidence goes now.
    fn history_released(&self);
}

impl WorthQueryProductBranchOwnerCleanupRecord {
    pub(super) fn release_retired_history(
        &mut self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        let Some(WorthQueryProductBranchHistoryCleanup { resume }) = self.history.as_mut() else {
            return Ok(());
        };
        let walked = walk(runtime, resume);
        // Every release, complete or not, is followed before it reports.
        runtime.owner_cleanup.history_released();
        walked?;
        if resume.is_empty() {
            self.history = None;
            Ok(())
        } else {
            Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)
        }
    }
}

/// A live fork, the live branch it forked from or an installed child keeps the
/// history it reaches; a walk ends there. A held read or a head still in
/// custody keeps only the commits from it down, and the walk resumes from it.
fn walk(
    runtime: &WorthQueryProductRuntime,
    resume: &mut Vec<CompositeCommitIdentity>,
) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
    let mut index = 0;
    while let Some(from) = resume.get(index) {
        let release = runtime
            .owner
            .lifecycle_port()
            .release_retired_branch_history(from)
            .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable)?;
        match release.held_commit() {
            Some(held) => {
                resume[index] = held.clone();
                index += 1;
            }
            None => {
                resume.remove(index);
            }
        }
    }
    Ok(())
}
