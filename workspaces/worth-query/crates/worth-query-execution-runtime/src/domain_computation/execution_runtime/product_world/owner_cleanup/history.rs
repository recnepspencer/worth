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
        let result = runtime
            .owner
            .recovery_port()
            .release_retired_branch_history(from);
        apply_release_result(resume, &mut index, result)?;
    }
    Ok(())
}

fn apply_release_result(
    resume: &mut Vec<CompositeCommitIdentity>,
    index: &mut usize,
    result: Result<
        worth_runtime_world::facade::RetiredBranchHistoryRelease,
        worth_runtime_world::facade::HistoryReclamationDenial,
    >,
) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
    let release = match result {
        Ok(release) => release,
        Err(worth_runtime_world::facade::HistoryReclamationDenial::PartialPublication {
            resume_from,
            ..
        }) => {
            apply_partial_resume(resume, *index, resume_from);
            return Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable);
        }
        Err(_) => {
            return Err(WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryUnavailable);
        }
    };
    match release.held_commit() {
        Some(held) => {
            resume[*index] = held.clone();
            *index += 1;
        }
        None => {
            resume.remove(*index);
        }
    }
    Ok(())
}

fn apply_partial_resume<T>(resume: &mut Vec<T>, index: usize, resume_from: Option<T>) {
    match resume_from {
        Some(next) => resume[index] = next,
        None => {
            resume.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_publication_advances_resume_cursor_and_retry_reclaims_suffix() {
        let mut resume = vec![4u64, 9u64];
        apply_partial_resume(&mut resume, 0, Some(2));
        assert_eq!(resume, vec![2, 9]);
        apply_partial_resume(&mut resume, 0, None);
        assert_eq!(resume, vec![9]);
    }
}
