//! Every named Store recovery stage must be reached and killed. Most cases
//! replay a commit killed after WAL durability. Cleanup cases kill a published
//! checkpoint before WAL reclamation, with small certification-only segments
//! and a bounded real redo prefix. Three recoveries, continuation and fresh
//! identity allocation must then agree with the same memory-only reference.

pub(crate) use {std::fs};
pub(crate) use {std::ffi::OsString, std::num::NonZeroU64, std::path::PathBuf};

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam};
pub(crate) use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaOperationRole};
pub(crate) use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint, worth_store::physical_runtime::PhysicalRecoveryYieldpointStage as Stage};
pub(crate) use {worth_store_relational::RelationalStoreCertificationProfile, worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeRequest};

pub(crate) use {crate::crash_oracle::assert_continues, crate::crash_oracle::assert_recovered, crate::crash_oracle::InFlight};
pub(crate) use {crate::durable_run::assert_holds, crate::durable_run::open, crate::durable_run::recorded_schema};
pub(crate) use {crate::killed_child, crate::killed_child::ChildEnd};
pub(crate) use {crate::process_kill, crate::process_kill::Stop, crate::process_kill::SEED};
pub(crate) use {crate::schema_transition};
pub(crate) use {crate::workload};

pub(crate) const STAGE_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_STAGE";
pub(crate) const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_LOCATION";
pub(crate) const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_MARKERS";
pub(crate) const RECOVERY_CHILD: &str = "recovery_interruption::recovery_interruption_child";

pub(crate) const STAGES: [Stage; 13] = [
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenRootManifest,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ObligationDischarge,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ServingHandoff,
];

// No wildcard: a new Store stage must extend the crash oracle's roster.
pub(crate) fn asserted_stage_index(stage: Stage) -> usize {
    match stage {
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization => 0,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization => 1,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateMaterialization => 2,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateSynchronization => 3,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement => 4,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization => 5,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector => 6,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenRootManifest => 7,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding => 8,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead => 9,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval => 10,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ObligationDischarge => 11,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ServingHandoff => 12,
    }
}

pub(crate) fn prove_stage(
    stage: Stage,
    script: &crate::workload::Script,
    execution: &ExecutionRequest<'_, '_>,
    wal_segment_bytes: Option<NonZeroU64>,
) {
    let case = format!("seed {SEED:#x} recovery killed at {}", stage.label());
    let (stop, in_flight) = if wal_segment_bytes.is_some() {
        (
            crate::process_kill::Stop::CheckpointAcknowledged(worth_store::physical_runtime::production::PhysicalCheckpointStep::NamespaceSynchronization),
            crate::crash_oracle::InFlight::Nothing,
        )
    } else {
        (crate::process_kill::Stop::Commit(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability), crate::crash_oracle::InFlight::Durable)
    };
    let killed = crate::process_kill::launch_with_wal_segment_bytes(stop, wal_segment_bytes);
    if stage == worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ObligationDischarge {
        leave_indeterminate_replacement(&killed.location, execution);
    }
    let markers = tempfile::tempdir().unwrap();
    let label = std::path::PathBuf::from(stage.label());
    let bytes = wal_segment_bytes.map(|bytes| std::path::PathBuf::from(bytes.get().to_string()));
    let mut environment: Vec<(&str, &std::path::Path)> = vec![
        (STAGE_ENV, &label),
        (LOCATION_ENV, &killed.location),
        (MARKERS_ENV, markers.path()),
    ];
    if let Some(bytes) = &bytes {
        environment.push((crate::process_kill::WAL_SEGMENT_BYTES_ENV, bytes));
    }
    let end = crate::killed_child::run_and_kill_observing(
        RECOVERY_CHILD,
        &environment,
        &markers.path().join("reached"),
        &markers.path().join("finished"),
        || crate::incarnation::assert_contended(&killed.location),
    );
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "required recovery stage was not killed: {}",
        stage.label()
    );
    eprintln!("killed recovery stage: {}", stage.label());

    let home = open(&killed.location, execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    let commits = assert_recovered(&home, script, &killed.acknowledged, &in_flight, &case);
    let recovered = crate::workload::artifacts_of(home.runtime());
    home.close();
    for attempt in ["second", "third"] {
        let again = open(&killed.location, execution)
            .unwrap_or_else(|refusal| panic!("{case}: {attempt} recovery: {refusal:?}"));
        assert_holds(&again, script, &recovered);
        if attempt == "third" {
            assert_continues(&killed.location, again, script, commits, execution, &case);
        } else {
            again.close();
        }
    }
}

/// A settled WAL seam leaves no physical-work obligation. Lose the answer
/// to a real root replacement so recovery must adjudicate and discharge one.
/// Checkpointing changes no logical history, even when its selection is unclear.
pub(crate) fn leave_indeterminate_replacement(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) {
    let fault = (
        worth_store::physical_runtime::certification::MediaOperationRole::AtomicReplace,
        std::num::NonZeroU64::new(1).unwrap(),
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let (home, activations) = crate::media_faults::open_with_faults(
        location,
        crate::schema_transition::TARGET,
        &[fault],
        execution,
    )
    .unwrap();
    crate::media_faults::arm(&activations);
    assert!(home.checkpoint(execution).is_err());
    assert!(activations[0].is_consumed());
    home.certification_abort();
}

pub(crate) fn recover_child_at(label: OsString) {
    let stage = worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::from_label(label.to_str().unwrap()).expect("a recovery stage label");
    let location = std::path::PathBuf::from(std::env::var_os(LOCATION_ENV).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS_ENV).unwrap());
    let yieldpoint = worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let opened = worth_store_relational::RelationalStoreHome::certification_open_with_recovery_yieldpoint(
        worth_store_relational::RelationalStoreHomeRequest::new(
            &location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        yieldpoint,
        &execution,
    );
    // The commit fixture includes a durable schema transition, so a completed
    // recovery serves its target. The checkpoint fixture still serves SOURCE.
    let home = match opened {
        Err(refusal) if recorded_schema(&refusal) == Some(crate::schema_transition::TARGET) => {
            open(&location, &execution)
        }
        opened => opened,
    };
    home.expect("a recovery that never parks completes").close();
    std::fs::write(markers.join("finished"), b"1").unwrap();
}