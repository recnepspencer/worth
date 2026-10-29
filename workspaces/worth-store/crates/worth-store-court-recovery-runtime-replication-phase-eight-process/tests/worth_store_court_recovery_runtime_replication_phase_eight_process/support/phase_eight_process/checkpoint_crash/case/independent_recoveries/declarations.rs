pub(crate) use {std::path::PathBuf};

pub(crate) use {worth_store_recovery_runtime::RecoveryReportEnvelope, worth_store_recovery_runtime::RecoveryReportOutcome};

pub(crate) use {crate::checkpoint_crash::evidence::snapshot::copy_directory, crate::checkpoint_crash::evidence::snapshot::snapshot_directory};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::process::recovery_report::fresh_recovery};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::case::crash_frontier::CrashObservation};

pub(crate) struct RecoverySet {
    pub(crate) crash: CrashObservation,
    pub(crate) first_root: PathBuf,
    pub(crate) second_root: PathBuf,
    pub(crate) first: RecoveryReportEnvelope,
}

pub(crate) fn run(crash: CrashObservation) -> RecoverySet {
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
            worth_store_recovery_runtime::RecoveryReportOutcome::Recovered,
            "an incomplete checkpoint candidate must lose to the valid selector"
        );
        assert_ne!(
            snapshot_directory(&first_root),
            crash.effect_snapshot,
            "recovery must settle the candidate residue while publishing the valid frontier"
        );
    }
    assert!(first.store_identity().is_some());
    if matches!(first.outcome(), worth_store_recovery_runtime::RecoveryReportOutcome::Recovered) {
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
    if matches!(first.outcome(), worth_store_recovery_runtime::RecoveryReportOutcome::Recovered) {
        assert!(
            first.counters().peak_recovery_bytes() > 0,
            "recovered checkpoint did not report its admitted recovery memory at {stage:?}"
        );
    }
    assert!(!matches!(
        first.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Refused | worth_store_recovery_runtime::RecoveryReportOutcome::PublicationIndeterminate
    ));

    RecoverySet {
        crash,
        first_root,
        second_root,
        first,
    }
}