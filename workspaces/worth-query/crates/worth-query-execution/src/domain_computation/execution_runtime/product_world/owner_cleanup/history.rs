use worth_runtime_world::facade::CompositeHistoryReclamationRequest;

use super::{
    WorthQueryProductBranchHistoryCleanup, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductBranchOwnerCleanupRecord,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryProductBranchOwnerCleanupRecord {
    pub(super) fn release_retired_history(
        &mut self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        let Some(history) = self.history.as_mut() else {
            return Ok(());
        };
        let released = match history {
            // A live fork or the live branch it forked from keeps the history
            // it reaches; the release ends there. A held read keeps only the
            // commits from it down, and the release resumes from it.
            WorthQueryProductBranchHistoryCleanup::RetiredBranch { resume } => {
                let release = runtime
                    .owner
                    .lifecycle_port()
                    .release_retired_branch_history(resume)
                    .map_err(|_| {
                        WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable
                    })?;
                match release.held_commit() {
                    Some(held) => {
                        *resume = held.clone();
                        false
                    }
                    None => true,
                }
            }
            WorthQueryProductBranchHistoryCleanup::ExactUnpublished { pending } => {
                release_exact_unpublished(runtime, pending)?
            }
        };
        if released {
            self.history = None;
            Ok(())
        } else {
            Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)
        }
    }
}

fn release_exact_unpublished(
    runtime: &WorthQueryProductRuntime,
    pending: &mut Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
) -> Result<bool, WorthQueryProductBranchOwnerCleanupDenial> {
    if pending.is_empty() {
        return Ok(true);
    }
    let outcome = runtime
        .owner
        .lifecycle_port()
        .reclaim_history(CompositeHistoryReclamationRequest::new(
            runtime.owner.owner_identity(),
            pending.clone(),
            pending.len(),
        ))
        .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable)?;
    let reclaimed = outcome.reclaimed_commits();
    let mut reclaimed_index = 0;
    pending.retain(|candidate| {
        if reclaimed.get(reclaimed_index) == Some(candidate) {
            reclaimed_index += 1;
            false
        } else {
            true
        }
    });
    Ok(pending.is_empty())
}
