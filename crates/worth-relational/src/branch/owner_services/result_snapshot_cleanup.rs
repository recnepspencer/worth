//! Request-bounded progress for one immutable result's original registration.
use super::owner_binding::RelationalOwnerServiceBinding;
use super::{RelationalCommitResultSnapshotRelease, RelationalCommitResultSnapshotReleaseDenial};
use crate::{runtime::RelationalRuntime, transactions::data::CommitResult};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

/// Retains the original result and its SAME release-cell retirement across
/// stops and lost responses. Final task/result/owner Drop is ordinary,
/// unbudgeted Rust destruction; only authenticated `advance` is bounded.
///
/// ```compile_fail
/// use worth_relational::facade::branch::RelationalCommitResultSnapshotCleanup;
/// fn cannot_construct_cleanup() {
///     let _ = RelationalCommitResultSnapshotCleanup {};
/// }
/// ```
#[derive(Debug)]
#[must_use]
pub struct RelationalCommitResultSnapshotCleanup {
    owner: RelationalOwnerServiceBinding,
    result: Arc<CommitResult>,
    removed_here: bool,
}

#[derive(Debug)]
pub enum RelationalCommitResultSnapshotCleanupPending {
    Advanced,
    Busy,
    Stopped(worth_execution::WorkCeilingDenial),
}

#[derive(Debug)]
pub enum RelationalCommitResultSnapshotCleanupDenial {
    NotOwner,
    OwnerBindingMismatch,
    Release(RelationalCommitResultSnapshotReleaseDenial),
}

#[derive(Debug)]
#[must_use]
pub enum RelationalCommitResultSnapshotCleanupProgress {
    Pending {
        task: RelationalCommitResultSnapshotCleanup,
        cause: RelationalCommitResultSnapshotCleanupPending,
    },
    Refused {
        task: RelationalCommitResultSnapshotCleanup,
        cause: RelationalCommitResultSnapshotCleanupDenial,
    },
    Complete {
        result: Arc<CommitResult>,
        release: RelationalCommitResultSnapshotRelease,
    },
}

impl RelationalCommitResultSnapshotCleanup {
    pub(super) fn new(owner: RelationalOwnerServiceBinding, result: Arc<CommitResult>) -> Self {
        Self {
            owner,
            result,
            removed_here: false,
        }
    }

    /// Perform one nonwaiting owner step under the original request. Home must
    /// retain this task/result through pending/refused edges and lend its actual
    /// immutable owner. An admitted runtime read copy cannot discharge custody.
    pub fn advance(
        mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> RelationalCommitResultSnapshotCleanupProgress {
        use RelationalCommitResultSnapshotCleanupDenial as Denial;
        use RelationalCommitResultSnapshotCleanupPending as Pending;
        if !owner.is_owner() {
            return self.refused(Denial::NotOwner);
        }
        if !self.owner.matches_owner(owner) {
            return self.refused(Denial::OwnerBindingMismatch);
        }
        if let Err(cause) = super::result_snapshot_release::validate(owner, &self.result) {
            return self.refused(Denial::Release(cause));
        }
        let original = self.result.snapshot_release_observation();
        match original.retirement_complete() {
            Ok(Some(true)) => return self.complete(),
            Ok(None) => return self.pending(Pending::Busy),
            Ok(Some(false)) => {}
            Err(()) => {
                return self.refused(Denial::Release(
                    RelationalCommitResultSnapshotReleaseDenial::RetirementInterrupted,
                ))
            }
        }
        if let Err(cause) = request.consult() {
            return self.pending(Pending::Stopped(cause));
        }
        if !original.was_released() {
            match owner.visibility.try_remove_original_result(original) {
                Ok(Some(removed)) => {
                    self.removed_here |= removed;
                    return self.pending(Pending::Advanced);
                }
                Ok(None) => return self.pending(Pending::Busy),
                Err(()) => {
                    return self.refused(Denial::Release(
                        RelationalCommitResultSnapshotReleaseDenial::RegistrationMismatch,
                    ))
                }
            }
        }
        match original.advance_retirement() {
            Ok(Some(true)) => self.complete(),
            Ok(Some(false)) => self.pending(Pending::Advanced),
            Ok(None) => self.pending(Pending::Busy),
            Err(()) => self.refused(Denial::Release(
                RelationalCommitResultSnapshotReleaseDenial::RetirementInterrupted,
            )),
        }
    }

    fn pending(
        self,
        cause: RelationalCommitResultSnapshotCleanupPending,
    ) -> RelationalCommitResultSnapshotCleanupProgress {
        RelationalCommitResultSnapshotCleanupProgress::Pending { task: self, cause }
    }
    fn refused(
        self,
        cause: RelationalCommitResultSnapshotCleanupDenial,
    ) -> RelationalCommitResultSnapshotCleanupProgress {
        RelationalCommitResultSnapshotCleanupProgress::Refused { task: self, cause }
    }
    fn complete(self) -> RelationalCommitResultSnapshotCleanupProgress {
        let release = if self.removed_here {
            RelationalCommitResultSnapshotRelease::Released(
                crate::visibility::RelationalSnapshotReleaseReceipt::new(
                    self.result.snapshot.snapshot_id(),
                ),
            )
        } else {
            RelationalCommitResultSnapshotRelease::AlreadyReleased {
                snapshot_id: self.result.snapshot.snapshot_id(),
            }
        };
        RelationalCommitResultSnapshotCleanupProgress::Complete {
            result: self.result,
            release,
        }
    }
}
