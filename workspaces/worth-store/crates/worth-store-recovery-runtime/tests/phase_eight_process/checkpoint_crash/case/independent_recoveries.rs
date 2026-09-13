use std::path::PathBuf;

use worth_store_recovery_runtime::{RecoveryReportEnvelope, RecoveryReportOutcome};

use super::super::evidence::{copy_directory, snapshot_directory};
use super::super::process::fresh_recovery;
use super::crash_frontier::CrashObservation;

pub(super) struct RecoverySet {
    pub(super) crash: CrashObservation,
    pub(super) first_root: PathBuf,
    pub(super) second_root: PathBuf,
    pub(super) first: RecoveryReportEnvelope,
}

pub(super) fn run(crash: CrashObservation) -> RecoverySet {
    let stage = crash.fixture.stage;
    let expected_outcome = crash.expected_outcome;
    let first_root = crash.fixture.parent.path().join("first-recovery-root");
    let second_root = crash.fixture.parent.path().join("second-recovery-root");
    copy_directory(&crash.fixture.root, &first_root);
    copy_directory(&crash.fixture.root, &second_root);
    let first = fresh_recovery(
        &crash.fixture.parent,
        &first_root,
        "first",
        &crash.fixture.operation_program.expected,
    );
    let second = fresh_recovery(
        &crash.fixture.parent,
        &second_root,
        "second",
        &crash.fixture.operation_program.expected,
    );
    assert_eq!(
        first.outcome(),
        expected_outcome,
        "wrong recovery fate at {stage:?}"
    );
    if crash.candidate_is_residue {
        assert_eq!(
            first.outcome(),
            RecoveryReportOutcome::Recovered,
            "an incomplete checkpoint candidate must lose to the valid selector"
        );
        assert_ne!(
            snapshot_directory(&first_root),
            crash.effect_snapshot,
            "recovery must settle the candidate residue while publishing the valid frontier"
        );
    }
    assert!(first.store_identity().is_some());
    if matches!(first.outcome(), RecoveryReportOutcome::Recovered) {
        assert!(first.root_generation().is_some());
    }
    assert_eq!(
        first.outcome(),
        second.outcome(),
        "reopen drift at {stage:?}"
    );
    assert_eq!(
        first.store_identity(),
        second.store_identity(),
        "store identity drift at {stage:?}"
    );
    assert_eq!(first.root_generation(), second.root_generation());
    assert_eq!(
        first.counters(),
        second.counters(),
        "recovery counters drift at {stage:?}"
    );
    assert_eq!(
        first.counters().cleanup_performed(),
        second.counters().cleanup_performed(),
        "cleanup completion counters drift at {stage:?}"
    );
    assert_eq!(
        first.counters().cleanup_deferred(),
        second.counters().cleanup_deferred(),
        "cleanup deferral counters drift at {stage:?}"
    );
    if matches!(first.outcome(), RecoveryReportOutcome::Recovered) {
        assert!(
            first.counters().peak_recovery_bytes() > 0,
            "recovered checkpoint did not report its admitted recovery memory at {stage:?}"
        );
    }
    assert!(!matches!(
        first.outcome(),
        RecoveryReportOutcome::Refused | RecoveryReportOutcome::PublicationIndeterminate
    ));

    RecoverySet {
        crash,
        first_root,
        second_root,
        first,
    }
}
