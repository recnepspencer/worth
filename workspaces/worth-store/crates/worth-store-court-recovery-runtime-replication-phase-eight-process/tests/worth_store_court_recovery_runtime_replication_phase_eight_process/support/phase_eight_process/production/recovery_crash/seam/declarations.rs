pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryYieldpointStage};
pub(crate) use {worth_store_recovery_runtime::RecoveryReportOutcome};

pub(crate) use {worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard};
pub(crate) use {crate::phase_eight_process::comparison};
pub(crate) use {crate::phase_eight_process::history::physical_history::ParentPhysicalHistory};
pub(crate) use {crate::phase_eight_process::production::harness::process::spawn_recovery_at_yieldpoint, crate::phase_eight_process::production::harness::ObserverProcess, crate::phase_eight_process::production::harness::ProcessWorld, crate::phase_eight_process::production::harness::RuntimeProcess};

pub(crate) const RECOVERY_SEAM_PROFILE: &str = "c8-phase8-fate-coverage-v1";
pub(crate) const RECOVERY_SEAM_MEMORY_BUDGET_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) fn run(world: &ProcessWorld, index: usize, stage: PhysicalRecoveryYieldpointStage) {
    let mut seam = crate::phase_eight_process::production::recovery_crash::seam::InterruptedSeam::prepare(world, index, stage);
    seam.interrupt();
    let recoveries = seam.recover_independently();
    seam.assert_recovery(recoveries);
}

pub(crate) struct InterruptedSeam<'a> {
    pub(crate) world: &'a ProcessWorld,
    pub(crate) index: usize,
    pub(crate) stage: PhysicalRecoveryYieldpointStage,
    pub(crate) root: PathBuf,
    pub(crate) baseline: ObserverProcess,
    pub(crate) cleanup_before: Option<crate::phase_eight_process::history::parent_oracle::cleanup_candidate::CleanupCandidateProof>,
}

impl<'a> InterruptedSeam<'a> {
    pub(crate) fn prepare(
        world: &'a ProcessWorld,
        index: usize,
        stage: PhysicalRecoveryYieldpointStage,
    ) -> Self {
        let root = world
            .writer
            .root
            .parent()
            .expect("writer root parent")
            .join(format!("recovery-seam-{index}-{}", stage.label()));
        copy_directory(&world.writer.root, &root);
        let baseline = world.observe_root(&root, &format!("recovery-{index}-baseline"));
        let cleanup_before = is_cleanup_stage(stage).then(|| {
            world
                .cleanup_candidate(&root)
                .unwrap_or_else(|error| panic!("cleanup candidate precondition failed: {error}"))
        });
        Self {
            world,
            index,
            stage,
            root,
            baseline,
            cleanup_before,
        }
    }

    pub(crate) fn interrupt(&mut self) {
        let parent = self.root.parent().expect("recovery seam parent");
        let report = parent.join(format!("recovery-{}-report.bin", self.index));
        let reached = parent.join(format!("recovery-{}-reached", self.index));
        let release = parent.join(format!("recovery-{}-release", self.index));
        let mut child = spawn_recovery_at_yieldpoint(
            &self.root,
            &report,
            self.world.parent_path(),
            self.stage,
            &reached,
            &release,
        );
        wait_for_reached(&mut child, &reached, self.stage);
        let status = child
            .kill_and_wait()
            .expect("wait for killed recovery seam child");
        assert!(!status.success(), "recovery must die at {:?}", self.stage);
        assert!(
            !report.exists(),
            "a killed recovery process must not publish a terminal report at {:?}",
            self.stage
        );
        let crash = self
            .world
            .observe_root(&self.root, &format!("recovery-{}-crash", self.index));
        assert_crash_observation(&self.baseline, &crash, self.stage);
    }

    pub(crate) fn recover_independently(&self) -> RecoveryPair {
        let first = self.world.recover_root_with_profile(
            &self.root,
            &format!("recovery-{}-first", self.index),
            RECOVERY_SEAM_PROFILE,
        );
        let first_observer = self.world.observe_root(
            &self.root,
            &format!("recovery-{}-first-observer", self.index),
        );
        let first_history = self.world.parent_history_after_recovery(&self.root);
        self.assert_runtime_observer(&first, &first_observer, &first_history);
        if let Some(before) = &self.cleanup_before {
            let transition = self
                .world
                .verify_cleanup_transition(&self.root, before)
                .unwrap_or_else(|error| {
                    panic!("ordinary cleanup transition was not exact: {error}")
                });
            assert!(
                first.report.counters().cleanup_performed() <= before.covered_segments,
                "ordinary cleanup report cannot exceed the exact raw candidate removal set"
            );
            assert_eq!(first.report.counters().cleanup_deferred(), 0);
            assert_eq!(
                transition.removed_covered.len() as u64,
                before.covered_segments
            );
            assert_eq!(transition.retained, before.retained);
        }
        let second_root = self
            .world
            .writer
            .root
            .parent()
            .expect("writer root parent")
            .join(format!(
                "recovery-seam-{}-second-{}",
                self.index,
                self.stage.label()
            ));
        copy_directory(&self.world.writer.root, &second_root);
        let second = self.world.recover_root_with_profile(
            &second_root,
            &format!("recovery-{}-second", self.index),
            RECOVERY_SEAM_PROFILE,
        );
        let second_observer = self.world.observe_root(
            &second_root,
            &format!("recovery-{}-second-observer", self.index),
        );
        let second_history = self.world.parent_history_after_recovery(&second_root);
        RecoveryPair {
            first,
            first_observer,
            first_history,
            second,
            second_observer,
            second_history,
            second_root,
        }
    }

    pub(crate) fn assert_recovery(&self, pair: RecoveryPair) {
        assert_eq!(
            pair.first.report.outcome(),
            worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
        );
        assert_eq!(
            pair.second.report.outcome(),
            worth_store_recovery_runtime::RecoveryReportOutcome::Recovered
        );
        assert!(pair.first.report.root_generation().is_some());
        assert!(
            pair.first.report.counters().peak_recovery_bytes() < RECOVERY_SEAM_MEMORY_BUDGET_BYTES
        );
        if self.has_partial_publication() {
            self.assert_partial_convergence(&pair);
        } else {
            assert_eq!(
                pair.first_history, pair.second_history,
                "recovery drift at {:?}",
                self.stage
            );
            crate::phase_eight_process::comparison::compare_independent_physical_evidence(
                &pair.first_observer.report,
                &pair.second_observer.report,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "independent recovery observer drift at {:?}: {error:?}",
                    self.stage
                )
            });
        }
        assert_eq!(
            pair.first.report.store_identity(),
            pair.second.report.store_identity(),
            "store identity drift at {:?}",
            self.stage
        );
        assert_eq!(
            pair.first.report.root_generation(),
            pair.second.report.root_generation(),
            "root generation drift at {:?}",
            self.stage
        );
        remove_recovered_roots(&self.root, &pair.second_root, self.stage);
    }

    pub(crate) fn assert_partial_convergence(&self, pair: &RecoveryPair) {
        let settled = self.world.recover_root_with_profile(
            &self.root,
            &format!("recovery-{}-settled", self.index),
            RECOVERY_SEAM_PROFILE,
        );
        let settled_observer = self.world.observe_root(
            &self.root,
            &format!("recovery-{}-settled-observer", self.index),
        );
        let settled_history = self.world.parent_history_after_recovery(&self.root);
        let converged = self.world.recover_root_with_profile(
            &self.root,
            &format!("recovery-{}-converged", self.index),
            RECOVERY_SEAM_PROFILE,
        );
        let converged_observer = self.world.observe_root(
            &self.root,
            &format!("recovery-{}-converged-observer", self.index),
        );
        let converged_history = self.world.parent_history_after_recovery(&self.root);
        assert_eq!(converged.report.outcome(), worth_store_recovery_runtime::RecoveryReportOutcome::Recovered);
        assert_eq!(settled_history, converged_history);
        crate::phase_eight_process::comparison::compare_independent_physical_evidence(
            &settled_observer.report,
            &converged_observer.report,
        )
        .unwrap_or_else(|error| {
            panic!(
                "partial publication observer drift at {:?}: {error:?}",
                self.stage
            )
        });
        self.assert_runtime_observer(&settled, &settled_observer, &settled_history);
        assert_eq!(
            pair.second.report.store_identity(),
            settled.report.store_identity()
        );
        assert_eq!(
            pair.second.report.root_generation(),
            settled.report.root_generation()
        );
    }

    pub(crate) fn assert_runtime_observer(
        &self,
        runtime: &RuntimeProcess,
        observer: &ObserverProcess,
        history: &ParentPhysicalHistory,
    ) {
        crate::phase_eight_process::comparison::compare_runtime_and_observer_with_budget(
            &runtime.report,
            &observer.report,
            history,
            RECOVERY_SEAM_MEMORY_BUDGET_BYTES,
        )
        .unwrap_or_else(|error| panic!("runtime/observer drift at {:?}: {error:?}", self.stage));
    }

    pub(crate) fn has_partial_publication(&self) -> bool {
        matches!(
            self.stage,
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenRootManifest
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead
                | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval
        )
    }
}

pub(crate) struct RecoveryPair {
    pub(crate) first: RuntimeProcess,
    pub(crate) first_observer: ObserverProcess,
    pub(crate) first_history: ParentPhysicalHistory,
    pub(crate) second: RuntimeProcess,
    pub(crate) second_observer: ObserverProcess,
    pub(crate) second_history: ParentPhysicalHistory,
    pub(crate) second_root: PathBuf,
}

pub(crate) fn assert_crash_observation(
    baseline: &ObserverProcess,
    crash: &ObserverProcess,
    stage: PhysicalRecoveryYieldpointStage,
) {
    if stage != worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval {
        assert!(
            crash.report.artifact_count() >= baseline.report.artifact_count(),
            "crash observer lost artifacts before cleanup removal at {:?}",
            stage
        );
    }
    assert!(crash.report.bytes_read() > 0);
    assert_ne!(crash.report.artifact_set_digest(), [0; 32]);
}

pub(crate) fn is_cleanup_stage(stage: PhysicalRecoveryYieldpointStage) -> bool {
    matches!(
        stage,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead
            | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval
    )
}

pub(crate) fn remove_recovered_roots(root: &Path, second_root: &Path, stage: PhysicalRecoveryYieldpointStage) {
    std::fs::remove_dir_all(root)
        .unwrap_or_else(|error| panic!("remove exact killed-writer root at {:?}: {error}", stage));
    std::fs::remove_dir_all(second_root).unwrap_or_else(|error| {
        panic!("remove exact second recovery root at {:?}: {error}", stage)
    });
}

pub(crate) fn wait_for_reached(
    child: &mut ProcessChildGuard,
    marker: &Path,
    stage: PhysicalRecoveryYieldpointStage,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !marker.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "recovery yieldpoint timeout at {:?}",
            stage
        );
        assert!(
            child
                .child_mut()
                .try_wait()
                .expect("poll recovery yieldpoint child")
                .is_none(),
            "recovery child exited before {:?}",
            stage
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create recovery seam root");
    for entry in std::fs::read_dir(source).expect("read recovery seam source") {
        let entry = entry.expect("read recovery seam entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).expect("copy recovery seam artifact");
        }
    }
}