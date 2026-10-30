pub(crate) use std::path::Path;

pub(crate) use tempfile::TempDir;
pub(crate) use {
    worth_store_offline_verifier::RecoveryObserverDecodeDenial,
    worth_store_offline_verifier::RecoveryObserverReport,
    worth_store_offline_verifier::RECOVERY_OBSERVER_REPORT_PROTOCOL,
};
pub(crate) use {
    worth_store_recovery_runtime::RecoveryReportDecodeDenial,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
    worth_store_recovery_runtime::RECOVERY_REPORT_PROTOCOL,
};

pub(crate) use crate::production::fate_markers::IndexedRecoveryFate;
pub(crate) use {crate::history, crate::phase_eight_process::comparison};

pub(crate) use crate::history::schedule::in_flight_mutation::MutationCrashWorkload;
pub(crate) use crate::process::support::production_harness::run_recovery_with_profile;
pub(crate) use {
    crate::process::support::production_harness::assert_child_succeeded,
    crate::process::support::production_harness::run_observer,
    crate::process::support::production_harness::spawn_recovery_at_yieldpoint,
    crate::process::support::production_harness::spawn_recovery_at_yieldpoint_with_deadline,
};
pub(crate) use {
    crate::production::harness::markers::RecoveryFateMarker,
    crate::production::harness::markers::RecoveryRuntimeMarker,
};

pub(crate) const C8_RECOVERY_MEMORY_BUDGET_BYTES: u64 = 512 * 1024;

pub(crate) struct ProcessWorld {
    pub(crate) parent: TempDir,
    pub(crate) writer: crate::process::support::history::lifecycle::KilledProductionWriter,
}

pub(crate) struct ObserverProcess {
    pub(crate) process_id: u32,
    pub(crate) report: RecoveryObserverReport,
    pub(crate) encoded: Vec<u8>,
}

pub(crate) struct RuntimeProcess {
    pub(crate) process_id: u32,
    pub(crate) marker: RecoveryRuntimeMarker,
    pub(crate) fates: RecoveryFateMarker,
    pub(crate) indexed_fates: Vec<IndexedRecoveryFate>,
    pub(crate) report: RecoveryReportEnvelope,
    pub(crate) encoded: Vec<u8>,
}

impl ProcessWorld {
    pub(crate) fn start_mutation_crash(
        stage: &'static str,
        workload: MutationCrashWorkload,
        schedule_seed: u64,
        perturbation_seed: u64,
    ) -> Self {
        let parent = tempfile::tempdir().expect("C8 mutation-crash process parent");
        let writer =
            crate::process::support::history::mutation_writer::launch_killed_mutation_writer(
                parent.path(),
                stage,
                workload,
                schedule_seed,
                perturbation_seed,
            )
            .expect("production writer must leave a killed mutation root");
        Self { parent, writer }
    }

    pub(crate) fn start_capacity_transition_crash(
        schedule_seed: u64,
        perturbation_seed: u64,
    ) -> Self {
        let parent = tempfile::tempdir().expect("C8 capacity-transition process parent");
        let writer = crate::process::support::history::mutation_writer::launch_killed_mutation_writer_with_operation_count(
            parent.path(),
            "during-root-publication",
            crate::history::schedule::in_flight_mutation::MutationCrashWorkload::CapacityTransition,
            schedule_seed,
            perturbation_seed,
            crate::history::DEFAULT_OPERATION_COUNT,
        )
        .expect("production writer must leave a killed capacity-transition root");
        Self { parent, writer }
    }

    pub(crate) fn start(stage: &str, schedule_seed: u64, perturbation_seed: u64) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 process parent");
        let writer =
            crate::process::support::history::checkpoint_writer::launch_killed_production_writer(
                parent.path(),
                stage,
                schedule_seed,
                perturbation_seed,
            )
            .expect("production writer must leave a killed persisted root");
        Self { parent, writer }
    }

    pub(crate) fn start_with_operation_count(
        stage: &str,
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 process parent");
        let writer = crate::process::support::history::checkpoint_writer::launch_killed_production_writer_with_operation_count(
            parent.path(),
            stage,
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .expect("production writer must leave a killed persisted root");
        Self { parent, writer }
    }

    pub(crate) fn start_durable_unacknowledged(schedule_seed: u64, perturbation_seed: u64) -> Self {
        Self::start_durable_unacknowledged_with_operation_count(
            schedule_seed,
            perturbation_seed,
            crate::history::DEFAULT_OPERATION_COUNT,
        )
    }

    pub(crate) fn start_durable_unacknowledged_with_operation_count(
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 durable-unacknowledged parent");
        let writer = crate::process::support::history::durable_before_ack_writer::launch_killed_durable_unacknowledged_writer_with_operation_count(
            parent.path(),
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .unwrap_or_else(|error| panic!("durable-before-ack writer fixture failed: {error}"));
        Self { parent, writer }
    }

    pub(crate) fn start_cleanup_world_with_operation_count(
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 cleanup-rotation parent");
        let writer = crate::process::support::history::cleanup_rotation_writer::launch_killed_cleanup_writer_with_operation_count(
            parent.path(),
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .unwrap_or_else(|error| panic!("cleanup-rotation writer fixture failed: {error}"));
        Self { parent, writer }
    }

    pub(crate) fn observe(&self, name: &str) -> ObserverProcess {
        self.observe_root(&self.writer.root, name)
    }

    pub(crate) fn observe_root(&self, root: &Path, name: &str) -> ObserverProcess {
        let output_path = self
            .parent
            .path()
            .join(format!("{name}-observer-report.bin"));
        let (process_id, output) = run_observer(root, output_path.clone(), self.parent.path());
        assert_child_succeeded(name, &output);
        let encoded = std::fs::read(&output_path).expect("observer report bytes");
        let report = worth_store_offline_verifier::RecoveryObserverReport::decode(&encoded)
            .expect("observer report decode");
        ObserverProcess {
            process_id,
            report,
            encoded,
        }
    }

    pub(crate) fn recover(&self, name: &str) -> RuntimeProcess {
        self.recover_root(&self.writer.root, name)
    }

    pub(crate) fn recover_root(&self, root: &Path, name: &str) -> RuntimeProcess {
        self.recover_root_with_profile(root, name, "c8-phase2-admission-v1")
    }

    pub(crate) fn recover_root_with_profile(
        &self,
        root: &Path,
        name: &str,
        profile: &str,
    ) -> RuntimeProcess {
        crate::phase_eight_process::production::harness::recovery::recover_root_with_profile(
            self, root, name, profile,
        )
    }

    pub(crate) fn parent_history(
        &self,
    ) -> crate::phase_eight_process::history::physical_history::ParentPhysicalHistory {
        self.parent_history_root(&self.writer.root)
    }

    pub(crate) fn parent_history_root(
        &self,
        root: &Path,
    ) -> crate::phase_eight_process::history::physical_history::ParentPhysicalHistory {
        crate::phase_eight_process::history::physical_history::ParentPhysicalHistory::capture(
            root,
            &self.writer.expected,
        )
        .expect("capture parent history")
    }

    pub(crate) fn parent_history_after_recovery(
        &self,
        root: &Path,
    ) -> crate::phase_eight_process::history::physical_history::ParentPhysicalHistory {
        crate::phase_eight_process::history::physical_history::ParentPhysicalHistory::capture_with_unresolved_record(root, &self.writer.expected)
            .expect("capture parent history after recovery")
    }

    pub(crate) fn require_cleanup_candidate(&self) -> Result<(), String> {
        crate::phase_eight_process::history::parent_oracle::cleanup_candidate::capture(
            &self.writer.root,
        )
        .map(|_| ())
    }

    pub(crate) fn cleanup_candidate(
        &self,
        root: &Path,
    ) -> Result<crate::phase_eight_process::history::parent_oracle::cleanup_candidate::CleanupCandidateProof, String>{
        crate::phase_eight_process::history::parent_oracle::cleanup_candidate::capture(root)
    }

    pub(crate) fn verify_cleanup_transition(
        &self,
        root: &Path,
        before: &crate::phase_eight_process::history::parent_oracle::cleanup_candidate::CleanupCandidateProof,
    ) -> Result<crate::phase_eight_process::history::parent_oracle::cleanup_candidate::CleanupTransitionProof, String>{
        crate::phase_eight_process::history::parent_oracle::cleanup_candidate::verify_removed_covered(root, before)
    }

    pub(crate) fn verify_cleanup_preserved(
        &self,
        root: &Path,
        before: &crate::phase_eight_process::history::parent_oracle::cleanup_candidate::CleanupCandidateProof,
    ) -> Result<(), String> {
        crate::phase_eight_process::history::parent_oracle::cleanup_candidate::verify_preserved(
            root, before,
        )
    }

    pub(crate) fn parent_path(&self) -> &Path {
        self.parent.path()
    }
}

pub(crate) fn compare_runtime_and_observer(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
    history: &crate::phase_eight_process::history::physical_history::ParentPhysicalHistory,
) {
    compare_runtime_and_observer_with_budget(
        runtime,
        observer,
        history,
        C8_RECOVERY_MEMORY_BUDGET_BYTES,
    );
}

pub(crate) fn compare_runtime_and_observer_with_budget(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
    history: &crate::phase_eight_process::history::physical_history::ParentPhysicalHistory,
    memory_budget_bytes: u64,
) {
    assert_eq!(
        runtime.report.outcome(),
        worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
    );
    assert!(runtime.fates.acknowledged > 0);
    assert!(runtime.report.store_identity().is_some());
    assert!(runtime.report.root_generation().is_some());
    assert!(runtime.report.counters().peak_recovery_bytes() > 0);
    assert!(
        runtime.report.counters().peak_recovery_bytes() < memory_budget_bytes,
        "recovery exceeded the ordinary process memory boundary"
    );
    assert!(observer.report.artifact_count() > 0);
    assert!(observer.report.bytes_read() > 0);
    assert_ne!(observer.report.artifact_set_digest(), [0; 32]);
    assert!(observer.report.artifact_identity_count() > 0);
    assert_ne!(observer.report.artifact_identity_digest(), [0; 32]);
    assert!(observer.report.generation_link_count() > 0);
    assert!(observer.report.durable_selector_count() > 0);
    assert_eq!(
        observer.report.selector_store_identity(),
        runtime.report.store_identity()
    );
    assert_eq!(
        observer.report.current_root_generation(),
        runtime.report.root_generation()
    );
    crate::phase_eight_process::comparison::compare_runtime_and_observer_with_budget(
        &runtime.report,
        &observer.report,
        history,
        memory_budget_bytes,
    )
    .expect("runtime and observer must agree on the recovered physical frontier");
}

pub(crate) fn assert_protocol_families_are_distinct(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
) {
    assert_ne!(
        RECOVERY_REPORT_PROTOCOL, RECOVERY_OBSERVER_REPORT_PROTOCOL,
        "recovery and observer reports must remain distinct protocol families"
    );
    assert!(matches!(
        worth_store_recovery_runtime::RecoveryReportEnvelope::decode(&observer.encoded),
        Err(worth_store_recovery_runtime::RecoveryReportDecodeDenial::WrongProtocolFamily)
    ));
    assert!(matches!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&runtime.encoded),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::WrongProtocolFamily)
    ));
}

pub(crate) fn assert_missing_root_observer_fails() {
    let parent = tempfile::tempdir().expect("missing-root process parent");
    let missing_root = parent.path().join("missing-root");
    let report = parent.path().join("missing-observer-report.bin");
    let (_, output) = run_observer(&missing_root, report.clone(), parent.path());
    assert!(!output.status.success());
    assert!(!report.exists());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("physical_store_offline_observer:"),
        "missing-root observer omitted its typed denial marker"
    );
}
