use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    WorthQueryProductBranchHistoryCleanup, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductBranchOwnerCleanupRecord,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryProductBranchOwnerCleanupRecord {
    pub(super) fn release_retired_history(
        &mut self,
        runtime: &WorthQueryProductRuntime,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        let Some(WorthQueryProductBranchHistoryCleanup { resume }) = self.history.as_mut() else {
            return Ok(());
        };
        let walked = walk(runtime, resume, execution);
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
    execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
    let mut index = 0;
    while let Some(from) = resume.get(index) {
        let port = runtime.owner.recovery_port();
        let release = match match execution {
            Some(execution) => port.release_retired_branch_history_with_request(from, execution),
            None => port.release_retired_branch_history(from),
        } {
            Ok(release) => release,
            Err(worth_runtime_world::facade::HistoryReclamationDenial::PartialPublication {
                resume_from,
                ..
            }) => {
                match resume_from {
                    Some(next) => resume[index] = next,
                    None => {
                        resume.remove(index);
                    }
                }
                return Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable);
            }
            Err(_) => {
                return Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable);
            }
        };
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
