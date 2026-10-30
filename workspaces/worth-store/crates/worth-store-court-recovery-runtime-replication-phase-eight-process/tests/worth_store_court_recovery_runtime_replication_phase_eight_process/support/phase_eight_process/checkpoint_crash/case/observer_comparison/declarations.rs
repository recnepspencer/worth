pub(crate) use worth_store_recovery_runtime::RecoveryReportOutcome;

pub(crate) use crate::checkpoint_crash::evidence::snapshot::snapshot_directory;
pub(crate) use crate::phase_eight_process::checkpoint_crash::case::independent_recoveries::RecoverySet;
pub(crate) use crate::process::support::checkpoint_crash::observer_report::fresh_observer;
pub(crate) use {crate::history, crate::phase_eight_process::comparison};

pub(crate) fn compare(recoveries: &RecoverySet) {
    let crash = &recoveries.crash;
    let stage = crash.fixture.stage;
    crash
        .crash_history
        .compare_report(&crash.crash_observer)
        .expect("parent history must match the dead-byte checkpoint observer");
    let baseline_artifacts = crash.fixture.baseline_snapshot.len() as u64;
    assert!(crash.crash_observer.artifact_count() >= baseline_artifacts);
    assert!(crash.crash_observer.bytes_read() > 0);
    assert_ne!(crash.crash_observer.artifact_set_digest(), [0; 32]);

    let observer = fresh_observer(
        &crash.fixture.parent,
        &recoveries.first_root,
        "observer-first",
    );
    let observer_reopen = fresh_observer(
        &crash.fixture.parent,
        &recoveries.first_root,
        "observer-reopen",
    );
    let independent_observer = fresh_observer(
        &crash.fixture.parent,
        &recoveries.second_root,
        "observer-independent",
    );
    assert!(
        observer.artifact_count() > 0,
        "observer lost {stage:?} tree"
    );
    assert!(
        observer.bytes_read() > 0,
        "observer read no {stage:?} bytes"
    );
    assert_ne!(observer.artifact_set_digest(), [0; 32]);
    assert_eq!(
        observer, observer_reopen,
        "same-root observer drift at {stage:?}"
    );
    let first_history = crate::phase_eight_process::history::physical_history::ParentPhysicalHistory::capture_with_unresolved_record(
        &recoveries.first_root,
        &crash.fixture.operation_program.expected,
    )
    .expect("capture parent history after checkpoint recovery");
    let second_history = crate::phase_eight_process::history::physical_history::ParentPhysicalHistory::capture_with_unresolved_record(
        &recoveries.second_root,
        &crash.fixture.operation_program.expected,
    )
    .expect("capture independent parent history after checkpoint recovery");
    second_history
        .compare_report(&independent_observer)
        .expect("parent history must match the independent observer");
    assert_eq!(
        first_history, second_history,
        "recovery roots diverged at {stage:?}"
    );
    if matches!(
        recoveries.first.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
    ) {
        assert_eq!(
            recoveries.first.root_generation(),
            first_history.current_root_generation(),
            "report root generation did not match persisted selector at {stage:?}"
        );
    }
    if matches!(
        crash.expected_outcome,
        worth_store_recovery_runtime::RecoveryReportOutcome::Blocked
    ) {
        assert_eq!(
            snapshot_directory(&recoveries.first_root),
            crash.effect_snapshot,
            "blocked recovery changed persisted bytes at {stage:?}"
        );
    }
    crate::phase_eight_process::comparison::compare_runtime_and_observer_with_budget(
        &recoveries.first,
        &observer,
        &first_history,
        4 * 1024 * 1024,
    )
    .unwrap_or_else(|disagreement| {
        panic!("runtime and observer evidence diverged at {stage:?}: {disagreement:?}")
    });
    crate::phase_eight_process::comparison::compare_independent_physical_evidence(
        &observer,
        &independent_observer,
    )
    .unwrap_or_else(|disagreement| {
        panic!("independent physical observer evidence diverged at {stage:?}: {disagreement:?}")
    });
    assert_eq!(
        snapshot_directory(&recoveries.first_root),
        snapshot_directory(&recoveries.second_root),
        "fresh recovery bytes drift at {stage:?}"
    );
}
