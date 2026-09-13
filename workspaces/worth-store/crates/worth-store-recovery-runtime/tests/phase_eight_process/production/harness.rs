use std::path::Path;

use tempfile::TempDir;
use worth_store_offline_verifier::{
    RecoveryObserverDecodeDenial, RecoveryObserverReport, RECOVERY_OBSERVER_REPORT_PROTOCOL,
};
use worth_store_recovery_runtime::{
    RecoveryReportDecodeDenial, RecoveryReportEnvelope, RecoveryReportOutcome,
    RECOVERY_REPORT_PROTOCOL,
};

use super::super::{comparison, history};
use super::fate_markers::IndexedRecoveryFate;

#[path = "harness/markers.rs"]
mod markers;
#[path = "harness/process.rs"]
mod process;
#[path = "harness/recovery.rs"]
mod recovery;

pub(super) use super::super::history::MutationCrashWorkload;
pub(super) use markers::{RecoveryFateMarker, RecoveryRuntimeMarker};
pub use process::run_recovery_with_profile;
pub(super) use process::{
    assert_child_succeeded, run_observer, spawn_recovery_at_yieldpoint,
    spawn_recovery_at_yieldpoint_with_deadline,
};

pub(super) const C8_RECOVERY_MEMORY_BUDGET_BYTES: u64 = 512 * 1024;

pub(super) struct ProcessWorld {
    parent: TempDir,
    pub(super) writer: history::KilledProductionWriter,
}

pub(super) struct ObserverProcess {
    pub(super) process_id: u32,
    pub(super) report: RecoveryObserverReport,
    pub(super) encoded: Vec<u8>,
}

pub(super) struct RuntimeProcess {
    pub(super) process_id: u32,
    pub(super) marker: RecoveryRuntimeMarker,
    pub(super) fates: RecoveryFateMarker,
    pub(super) indexed_fates: Vec<IndexedRecoveryFate>,
    pub(super) report: RecoveryReportEnvelope,
    pub(super) encoded: Vec<u8>,
}

impl ProcessWorld {
    pub(super) fn start_mutation_crash(
        stage: &'static str,
        workload: MutationCrashWorkload,
        schedule_seed: u64,
        perturbation_seed: u64,
    ) -> Self {
        let parent = tempfile::tempdir().expect("C8 mutation-crash process parent");
        let writer = history::launch_killed_mutation_writer(
            parent.path(),
            stage,
            workload,
            schedule_seed,
            perturbation_seed,
        )
        .expect("production writer must leave a killed mutation root");
        Self { parent, writer }
    }

    pub(super) fn start_capacity_transition_crash(
        schedule_seed: u64,
        perturbation_seed: u64,
    ) -> Self {
        let parent = tempfile::tempdir().expect("C8 capacity-transition process parent");
        let writer = history::launch_killed_mutation_writer_with_operation_count(
            parent.path(),
            "during-root-publication",
            MutationCrashWorkload::CapacityTransition,
            schedule_seed,
            perturbation_seed,
            super::super::history::DEFAULT_OPERATION_COUNT,
        )
        .expect("production writer must leave a killed capacity-transition root");
        Self { parent, writer }
    }

    pub(super) fn start(stage: &str, schedule_seed: u64, perturbation_seed: u64) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 process parent");
        let writer = history::launch_killed_production_writer(
            parent.path(),
            stage,
            schedule_seed,
            perturbation_seed,
        )
        .expect("production writer must leave a killed persisted root");
        Self { parent, writer }
    }

    pub(super) fn start_with_operation_count(
        stage: &str,
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 process parent");
        let writer = history::launch_killed_production_writer_with_operation_count(
            parent.path(),
            stage,
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .expect("production writer must leave a killed persisted root");
        Self { parent, writer }
    }

    pub(super) fn start_durable_unacknowledged(schedule_seed: u64, perturbation_seed: u64) -> Self {
        Self::start_durable_unacknowledged_with_operation_count(
            schedule_seed,
            perturbation_seed,
            super::super::history::DEFAULT_OPERATION_COUNT,
        )
    }

    pub(super) fn start_durable_unacknowledged_with_operation_count(
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 durable-unacknowledged parent");
        let writer = history::launch_killed_durable_unacknowledged_writer_with_operation_count(
            parent.path(),
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .unwrap_or_else(|error| panic!("durable-before-ack writer fixture failed: {error}"));
        Self { parent, writer }
    }

    pub(super) fn start_cleanup_world_with_operation_count(
        schedule_seed: u64,
        perturbation_seed: u64,
        operation_count: usize,
    ) -> Self {
        let parent = tempfile::tempdir().expect("Phase 8 cleanup-rotation parent");
        let writer = history::launch_killed_cleanup_writer_with_operation_count(
            parent.path(),
            schedule_seed,
            perturbation_seed,
            operation_count,
        )
        .unwrap_or_else(|error| panic!("cleanup-rotation writer fixture failed: {error}"));
        Self { parent, writer }
    }

    pub(super) fn observe(&self, name: &str) -> ObserverProcess {
        self.observe_root(&self.writer.root, name)
    }

    pub(super) fn observe_root(&self, root: &Path, name: &str) -> ObserverProcess {
        let output_path = self
            .parent
            .path()
            .join(format!("{name}-observer-report.bin"));
        let (process_id, output) = run_observer(root, output_path.clone(), self.parent.path());
        assert_child_succeeded(name, &output);
        let encoded = std::fs::read(&output_path).expect("observer report bytes");
        let report = RecoveryObserverReport::decode(&encoded).expect("observer report decode");
        ObserverProcess {
            process_id,
            report,
            encoded,
        }
    }

    pub(super) fn recover(&self, name: &str) -> RuntimeProcess {
        self.recover_root(&self.writer.root, name)
    }

    pub(super) fn recover_root(&self, root: &Path, name: &str) -> RuntimeProcess {
        self.recover_root_with_profile(root, name, "c8-phase2-admission-v1")
    }

    pub(super) fn recover_root_with_profile(
        &self,
        root: &Path,
        name: &str,
        profile: &str,
    ) -> RuntimeProcess {
        recovery::recover_root_with_profile(self, root, name, profile)
    }

    pub(super) fn parent_history(&self) -> history::ParentPhysicalHistory {
        self.parent_history_root(&self.writer.root)
    }

    pub(super) fn parent_history_root(&self, root: &Path) -> history::ParentPhysicalHistory {
        history::ParentPhysicalHistory::capture(root, &self.writer.expected)
            .expect("capture parent history")
    }

    pub(super) fn parent_history_after_recovery(
        &self,
        root: &Path,
    ) -> history::ParentPhysicalHistory {
        history::ParentPhysicalHistory::capture_with_unresolved_record(root, &self.writer.expected)
            .expect("capture parent history after recovery")
    }

    pub(super) fn require_cleanup_candidate(&self) -> Result<(), String> {
        history::capture_cleanup_candidate(&self.writer.root).map(|_| ())
    }

    pub(super) fn cleanup_candidate(
        &self,
        root: &Path,
    ) -> Result<history::CleanupCandidateProof, String> {
        history::capture_cleanup_candidate(root)
    }

    pub(super) fn verify_cleanup_transition(
        &self,
        root: &Path,
        before: &history::CleanupCandidateProof,
    ) -> Result<history::CleanupTransitionProof, String> {
        history::verify_cleanup_transition(root, before)
    }

    pub(super) fn verify_cleanup_preserved(
        &self,
        root: &Path,
        before: &history::CleanupCandidateProof,
    ) -> Result<(), String> {
        history::verify_cleanup_preserved(root, before)
    }

    pub(super) fn parent_path(&self) -> &Path {
        self.parent.path()
    }
}

pub(super) fn compare_runtime_and_observer(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
    history: &history::ParentPhysicalHistory,
) {
    compare_runtime_and_observer_with_budget(
        runtime,
        observer,
        history,
        C8_RECOVERY_MEMORY_BUDGET_BYTES,
    );
}

pub(super) fn compare_runtime_and_observer_with_budget(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
    history: &history::ParentPhysicalHistory,
    memory_budget_bytes: u64,
) {
    assert_eq!(runtime.report.outcome(), RecoveryReportOutcome::Recovered);
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
    comparison::compare_runtime_and_observer_with_budget(
        &runtime.report,
        &observer.report,
        history,
        memory_budget_bytes,
    )
    .expect("runtime and observer must agree on the recovered physical frontier");
}

pub(super) fn assert_protocol_families_are_distinct(
    runtime: &RuntimeProcess,
    observer: &ObserverProcess,
) {
    assert_ne!(
        RECOVERY_REPORT_PROTOCOL, RECOVERY_OBSERVER_REPORT_PROTOCOL,
        "recovery and observer reports must remain distinct protocol families"
    );
    assert!(matches!(
        RecoveryReportEnvelope::decode(&observer.encoded),
        Err(RecoveryReportDecodeDenial::WrongProtocolFamily)
    ));
    assert!(matches!(
        RecoveryObserverReport::decode(&runtime.encoded),
        Err(RecoveryObserverDecodeDenial::WrongProtocolFamily)
    ));
}

pub(super) fn assert_missing_root_observer_fails() {
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
