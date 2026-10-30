//! Two ways a media change reaches cleanup's own byte-exact revalidation
//! under C.11 Phase 6, where Store rereads the selected checkpoint before
//! cleanup (residue gate) and the selected media after it (rejoin):
//! - a checkpoint change made after the residue gate and left in place, the
//!   check-then-act race (`finish_changed_before_revalidation`);
//! - a WAL change made before `finish`, which the residue gate never reads.
//!
//! Both settle cleanup first, then Store's rejoin denies the changed media,
//! and the indeterminate outcome names the cleanup debt it left behind.

pub(crate) use std::cell::Cell;
pub(crate) use std::io::Write;
pub(crate) use std::rc::Rc;
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial;
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPublicationIndeterminate,
    worth_store_recovery_runtime::RecoveryCleanupEvidence,
    worth_store_recovery_runtime::RecoveryCleanupPosture,
    worth_store_recovery_runtime::ReopenedPhysicalRecovery,
};

/// Applies `change` after Store's residue gate and before cleanup
/// revalidates; nothing restores it.
pub(crate) fn finish_changed_before_revalidation(
    reopened: ReopenedPhysicalRecovery,
    change: impl FnOnce() + 'static,
) -> PhysicalRecoveryOutcome {
    let ran = std::rc::Rc::new(std::cell::Cell::new(false));
    let observed = std::rc::Rc::clone(&ran);
    let outcome =
        reopened.certification_finish_with_change_before_cleanup_revalidation(move || {
            change();
            observed.set(true);
        });
    assert!(ran.get(), "the change must run before cleanup revalidation");
    outcome
}

/// Store's rejoin denied the changed media after cleanup settled.
pub(crate) fn rejoin_denied(
    outcome: PhysicalRecoveryOutcome,
) -> PhysicalRecoveryPublicationIndeterminate {
    let indeterminate = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
            indeterminate,
        ) => indeterminate,
        other => panic!(
            "changed selected media must deny Store's rejoin, got {}",
            outcome_name(&other)
        ),
    };
    assert_eq!(
        indeterminate.handoff_failure(),
        Some(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
    );
    assert!(indeterminate.reopen_failure().is_none());
    assert!(!indeterminate.checkpoint_residue_indeterminate());
    assert_eq!(indeterminate.recovery_effects(), 0);
    *indeterminate
}

/// The cleanup debt settled before the rejoin, still named on the outcome.
pub(crate) fn deferred_cleanup(
    indeterminate: &PhysicalRecoveryPublicationIndeterminate,
) -> &RecoveryCleanupEvidence {
    let Some(worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence)) =
        indeterminate.cleanup_posture()
    else {
        panic!("settled cleanup debt must stay named on the indeterminate outcome")
    };
    evidence
}

pub(crate) fn outcome_name(outcome: &PhysicalRecoveryOutcome) -> &'static str {
    match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_) => "Recovered",
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(_) => "Refused",
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(_) => "Blocked",
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(_) => {
            "PublicationIndeterminate"
        }
    }
}

pub(crate) fn checkpoint_path(root: &Path) -> PathBuf {
    root.join("families").join("checkpoint.current")
}

pub(crate) fn append_byte(path: &Path) {
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(&[0xff])
        .unwrap();
}

pub(crate) fn flip_first_byte(path: &Path) {
    flip_byte(path, |bytes| &mut bytes[0]);
}

pub(crate) fn flip_last_byte(path: &Path) {
    flip_byte(path, |bytes| {
        bytes.last_mut().expect("artifact is nonempty")
    });
}

/// Rewrites an existing artifact in place; never creates one.
pub(crate) fn flip_byte(path: &Path, byte: impl FnOnce(&mut [u8]) -> &mut u8) {
    let mut bytes = std::fs::read(path).unwrap();
    *byte(&mut bytes) ^= 0xff;
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap()
        .write_all(&bytes)
        .unwrap();
}
