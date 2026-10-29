pub(crate) use {worth_store_offline_verifier::RecoveryObserverReport};
pub(crate) use {worth_store_recovery_runtime::RecoveryReportOutcome};

pub(crate) use {crate::history};
pub(crate) use {crate::checkpoint_crash::evidence::snapshot::assert_snapshot_preserved, crate::phase_eight_process::checkpoint_crash::evidence::frontier::assert_stage_frontier, crate::phase_eight_process::checkpoint_crash::evidence::frontier::derive_expected_frontier, crate::checkpoint_crash::evidence::snapshot::snapshot_directory};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::process::observer_report::fresh_observer, crate::phase_eight_process::checkpoint_crash::process::writer_process::wait_for_marker};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::case::fixture_setup::CheckpointFixture};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::case::oracle::independently_classify_in_flight};

pub(crate) struct CrashObservation {
    pub(crate) fixture: CheckpointFixture,
    pub(crate) effect_snapshot: std::collections::BTreeMap<String, (u64, [u8; 32])>,
    pub(crate) expected_outcome: RecoveryReportOutcome,
    pub(crate) candidate_is_residue: bool,
    pub(crate) crash_observer: RecoveryObserverReport,
    pub(crate) crash_history: crate::phase_eight_process::history::physical_history::ParentPhysicalHistory,
}

pub(crate) fn observe(mut fixture: CheckpointFixture) -> CrashObservation {
    let stage = fixture.stage;
    let start_marker = fixture.parent.path().join("checkpoint-start");
    let reached_marker = fixture.parent.path().join("checkpoint-reached");
    std::fs::write(&start_marker, stage.label()).expect("release checkpoint writer");
    let checkpoint_effect = format!(
        "checkpoint effect for scenario {} ({}) at {}",
        fixture.scenario_index,
        fixture.scenario.id,
        stage.label(),
    );
    wait_for_marker(&mut fixture.child, &reached_marker, &checkpoint_effect);

    let effect_snapshot = snapshot_directory(&fixture.root);
    let expected_frontier = derive_expected_frontier(stage.label());
    assert_stage_frontier(
        stage.label(),
        &fixture.root,
        &fixture.baseline_snapshot,
        &effect_snapshot,
        &expected_frontier,
    );
    let expected_outcome = expected_frontier.outcome();
    let candidate_is_residue = expected_frontier.candidate_is_residue();

    let killed = fixture
        .child
        .kill_and_wait()
        .expect("wait for killed checkpoint writer");
    assert!(!killed.success(), "writer must be killed at {stage:?}");

    let crash_observer = fresh_observer(&fixture.parent, &fixture.root, "crash-observer");
    if candidate_is_residue {
        assert!(
            crash_observer.residue_artifact_count()
                > fixture.baseline_observer.residue_artifact_count(),
            "{scenario:?} did not expose a new residue artifact after process death",
            scenario = fixture.scenario
        );
        assert!(
            crash_observer.residue_bytes() > fixture.baseline_observer.residue_bytes(),
            "{scenario:?} did not expose residue bytes after process death",
            scenario = fixture.scenario
        );
        assert_ne!(
            crash_observer.residue_digest(),
            fixture.baseline_observer.residue_digest(),
            "{scenario:?} residue digest did not change after process death",
            scenario = fixture.scenario
        );
    }
    assert_snapshot_preserved(
        &fixture.root,
        &fixture.baseline_snapshot,
        &effect_snapshot,
        stage.label(),
    );
    let crash_history =
        crate::phase_eight_process::history::physical_history::ParentPhysicalHistory::capture(&fixture.root, &fixture.operation_program.expected)
            .expect("capture parent history at killed checkpoint");
    let in_flight_fate = fixture
        .operation_program
        .expected
        .classify_in_flight_from_physical_artifacts(&fixture.root)
        .expect("classify in-flight mutation fate from selected physical evidence");
    let independent_fate =
        independently_classify_in_flight(&fixture.root, &fixture.operation_program.expected);
    assert_eq!(
        in_flight_fate, independent_fate,
        "in-flight fate reducer disagreed with raw persisted evidence at {stage:?}"
    );
    let baseline_artifacts = fixture.baseline_snapshot.len() as u64;
    assert!(crash_observer.artifact_count() >= baseline_artifacts);
    assert!(crash_observer.bytes_read() > 0);
    assert_ne!(crash_observer.artifact_set_digest(), [0; 32]);
    assert!(
        crash_history.checkpoint_count() >= 1,
        "the crash fixture must retain a checkpoint artifact at {stage:?}; observed {}",
        crash_history.checkpoint_count()
    );
    assert!(
        crash_history.latest_checkpoint_sequence() >= 2,
        "the crash fixture must cross two checkpoint intervals at {stage:?}; observed sequence {}",
        crash_history.latest_checkpoint_sequence()
    );
    assert!(
        crash_history.wal_segment_count() >= 2,
        "the crash fixture must rotate the WAL before {stage:?}; observed {} WAL segments",
        crash_history.wal_segment_count()
    );

    CrashObservation {
        fixture,
        effect_snapshot,
        expected_outcome,
        candidate_is_residue,
        crash_observer,
        crash_history,
    }
}