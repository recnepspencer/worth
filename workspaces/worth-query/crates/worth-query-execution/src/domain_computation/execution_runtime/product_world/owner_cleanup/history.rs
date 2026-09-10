use std::num::NonZeroUsize;

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
        let pending = match history {
            WorthQueryProductBranchHistoryCleanup::RetiredBranch {
                retired_head,
                retirement_boundary,
                pending,
            } => {
                if pending.is_none() {
                    *pending = Some(collect_retired_history(
                        runtime,
                        retired_head,
                        retirement_boundary,
                    )?);
                }
                pending
                    .as_mut()
                    .expect("retired history candidates were initialized")
            }
            WorthQueryProductBranchHistoryCleanup::ExactUnpublished { pending } => pending,
        };
        if pending.is_empty() {
            self.history = None;
            return Ok(());
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
        if pending.is_empty() {
            self.history = None;
            Ok(())
        } else {
            Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained)
        }
    }
}

fn collect_retired_history(
    runtime: &WorthQueryProductRuntime,
    retired_head: &worth_runtime_world::facade::CompositeCommitIdentity,
    retirement_boundary: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> Result<
    Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
    WorthQueryProductBranchOwnerCleanupDenial,
> {
    if retired_head == retirement_boundary {
        return Ok(Vec::new());
    }
    let installed = runtime
        .owner
        .inspection_port()
        .history_snapshot()
        .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable)?
        .installed_commits();
    let maximum = NonZeroUsize::new(installed)
        .ok_or(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable)?;
    let traversal = runtime
        .owner
        .inspection_port()
        .trace_ancestry(retired_head.clone(), maximum)
        .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable)?;
    let mut candidates = Vec::new();
    let mut reached_boundary = false;
    for commit in traversal.commits() {
        if commit.identity() == retirement_boundary {
            reached_boundary = true;
            break;
        }
        candidates.push(commit.identity().clone());
    }
    if !reached_boundary {
        return Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable);
    }
    Ok(candidates)
}
