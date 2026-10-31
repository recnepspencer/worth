//! Explicit same-result cleanup through the actual published snapshot owner.
use crate::runtime::RelationalRuntime;
use crate::transactions::data::CommitResult;
use crate::visibility::{RelationalSnapshotReleaseDenial, RelationalSnapshotReleaseReceipt};

#[derive(Debug)]
pub enum RelationalCommitResultSnapshotRelease {
    Released(RelationalSnapshotReleaseReceipt),
    AlreadyReleased {
        snapshot_id: crate::snapshots::data::SnapshotId,
    },
}
#[derive(Debug)]
pub enum RelationalCommitResultSnapshotReleaseDenial {
    Owner(crate::branch::RelationalBranchBasisDenial),
    Snapshot(RelationalSnapshotReleaseDenial),
    ResultBindingMismatch,
    RegistrationMismatch,
    RetirementInterrupted,
}
pub(super) fn release(
    runtime: &RelationalRuntime,
    result: &CommitResult,
) -> Result<RelationalCommitResultSnapshotRelease, RelationalCommitResultSnapshotReleaseDenial> {
    use RelationalCommitResultSnapshotRelease as Outcome;
    use RelationalCommitResultSnapshotReleaseDenial as Denial;
    validate(runtime, result)?;
    let handle = &result.snapshot;
    let original = result.snapshot_release_observation();
    match runtime
        .visibility
        .published_release_observation(handle.snapshot_id())
    {
        Some(current) if !std::sync::Arc::ptr_eq(&current, original) => {
            return Err(Denial::RegistrationMismatch)
        }
        None if original.was_released() => {
            if !original.finish_retirement_blocking() {
                return Err(Denial::RetirementInterrupted);
            }
            return Ok(Outcome::AlreadyReleased {
                snapshot_id: handle.snapshot_id(),
            });
        }
        None => {
            return Err(Denial::Snapshot(
                RelationalSnapshotReleaseDenial::UnknownSnapshot,
            ))
        }
        Some(_) => {}
    }
    match runtime.snapshots().release_snapshot(handle) {
        Ok(receipt) => Ok(Outcome::Released(receipt)),
        Err(RelationalSnapshotReleaseDenial::UnknownSnapshot) if original.was_released() => {
            if !original.finish_retirement_blocking() {
                return Err(Denial::RetirementInterrupted);
            }
            Ok(Outcome::AlreadyReleased {
                snapshot_id: handle.snapshot_id(),
            })
        }
        Err(cause) => Err(Denial::Snapshot(cause)),
    }
}

pub(super) fn validate(
    runtime: &RelationalRuntime,
    result: &CommitResult,
) -> Result<(), RelationalCommitResultSnapshotReleaseDenial> {
    use RelationalCommitResultSnapshotReleaseDenial as Denial;
    let handle = &result.snapshot;
    if handle.runtime_instance_id() != runtime.runtime_instance_id() {
        return Err(Denial::Snapshot(
            RelationalSnapshotReleaseDenial::ForeignRuntime {
                expected_runtime_instance_id: runtime.runtime_instance_id(),
                actual_runtime_instance_id: handle.runtime_instance_id(),
            },
        ));
    }
    if handle.branch_id() != &result.commit.branch_id
        || handle.version_id() != result.commit.version_id
        || result.version_id != result.commit.version_id
        || result.semantic_result().receipt() != &result.commit
    {
        return Err(Denial::ResultBindingMismatch);
    }
    let original = result.snapshot_release_observation();
    if !original.matches(handle) || !runtime.visibility.handles.owns_published_release(original) {
        return Err(Denial::RegistrationMismatch);
    }
    Ok(())
}
