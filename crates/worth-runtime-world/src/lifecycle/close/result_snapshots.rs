//! Exact World-owned result custody is retained across each Native cleanup call.
use std::sync::Arc;
use worth_relational::facade::branch::{
    RelationalBranchBasisPort, RelationalCommitResultSnapshotRelease as Released,
    RelationalCommitResultSnapshotReleaseDenial,
};
use worth_relational::facade::transactions::CommitResult;

#[derive(Debug)]
pub struct RuntimeWorldResultSnapshotCleanup {
    native: RelationalBranchBasisPort,
    results: Vec<Arc<CommitResult>>,
    next: usize,
    released: usize,
    already_released: usize,
}

#[derive(Debug)]
pub struct RuntimeWorldResultSnapshotCleanupReport {
    released: usize,
    already_released: usize,
}
impl RuntimeWorldResultSnapshotCleanupReport {
    pub fn released(&self) -> usize {
        self.released
    }
    pub fn already_released(&self) -> usize {
        self.already_released
    }
}

impl RuntimeWorldResultSnapshotCleanup {
    pub(in crate::lifecycle) fn from_closed_owner(
        native: RelationalBranchBasisPort,
        results: Vec<Arc<CommitResult>>,
    ) -> Self {
        Self {
            native,
            results,
            next: 0,
            released: 0,
            already_released: 0,
        }
    }

    /// Borrow the retained task. Unwind or refusal leaves the same result at
    /// the cursor; Native's actual registration fact resolves a lost response.
    pub fn advance(
        &mut self,
    ) -> Result<RuntimeWorldResultSnapshotCleanupReport, RelationalCommitResultSnapshotReleaseDenial>
    {
        while let Some(result) = self.results.get(self.next) {
            match self.native.release_commit_result_snapshot(result)? {
                Released::Released(_) => self.released += 1,
                Released::AlreadyReleased { .. } => self.already_released += 1,
            }
            self.next += 1;
        }
        Ok(RuntimeWorldResultSnapshotCleanupReport {
            released: self.released,
            already_released: self.already_released,
        })
    }
}
