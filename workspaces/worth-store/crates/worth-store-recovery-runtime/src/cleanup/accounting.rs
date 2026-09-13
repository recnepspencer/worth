use worth_store::physical_runtime::{
    PhysicalRecoveryCleanupFreshnessReadDenialKind, PhysicalRecoveryCleanupRemovalDenialKind,
    PhysicalRecoveryCleanupRemovalIndeterminate, RecoveryCleanupArtifactRevalidationDenial,
    RecoveryCleanupArtifactRevalidationProgress, StoreRecoveryCleanupFreshnessFailure,
    StoreRecoveryCleanupFreshnessSample,
};

use crate::handoff::{
    RecoveryCleanupCounters, RecoveryCleanupDeferralEvidence, RecoveryCleanupEvidence,
    RecoveryCleanupEvidenceParts, RecoveryCleanupPosture,
};

use super::{PerformedRecoveryCleanupRemoval, RecoveryCleanupDispositionKind, RecoveryCleanupPlan};

pub(super) struct RecoveryCleanupAccounting {
    plan_identity: [u8; 32],
    published_generation: u64,
    freshness: Vec<StoreRecoveryCleanupFreshnessSample>,
    performed: Vec<PerformedRecoveryCleanupRemoval>,
    deferrals: Vec<RecoveryCleanupDeferralEvidence>,
    counters: RecoveryCleanupCounters,
}

impl RecoveryCleanupAccounting {
    pub(super) fn begin(plan: &RecoveryCleanupPlan, terminal_binding_evaluations: u64) -> Self {
        let bytes_planned = plan
            .candidates()
            .iter()
            .map(|candidate| candidate.byte_count())
            .sum();
        Self {
            plan_identity: plan.identity(),
            published_generation: plan.published_generation(),
            freshness: Vec::new(),
            performed: Vec::new(),
            deferrals: Vec::new(),
            counters: RecoveryCleanupCounters {
                dispositions: plan.dispositions().len() as u64,
                actions_planned: plan.candidates().len() as u64,
                bytes_planned,
                terminal_binding_evaluations,
                ..RecoveryCleanupCounters::default()
            },
        }
    }

    pub(super) fn record_freshness_sample(&mut self, sample: StoreRecoveryCleanupFreshnessSample) {
        self.counters.freshness_evaluations += 1;
        self.counters.freshness_reads_completed += 1;
        self.counters.freshness_bytes_read += sample.selector_read().bytes().len() as u64;
        self.record_freshness_scheduler(true, false, false, true);
        self.freshness.push(sample);
    }

    pub(super) fn record_freshness_failure(
        &mut self,
        failure: &StoreRecoveryCleanupFreshnessFailure,
    ) {
        self.counters.freshness_evaluations += 1;
        if let Some(sample) = failure.sample() {
            self.counters.freshness_reads_completed += 1;
            self.counters.freshness_bytes_read += sample.selector_read().bytes().len() as u64;
            self.record_freshness_scheduler(true, false, false, true);
            self.freshness.push(sample.clone());
            return;
        }
        let Some(read) = failure.read() else {
            return;
        };
        if read.completed().is_some() {
            self.counters.freshness_reads_completed += 1;
            self.counters.freshness_bytes_read +=
                worth_store_physical_format::ROOT_SELECTOR_BYTES as u64;
        } else if read.denied().is_some() {
            self.counters.freshness_reads_denied += 1;
        }
        match read.kind() {
            PhysicalRecoveryCleanupFreshnessReadDenialKind::Admission(admission) => {
                self.record_freshness_scheduler(
                    admission.submission_recorded(),
                    admission.scheduler_deferred(),
                    admission.cancellation_recorded(),
                    false,
                );
            }
            PhysicalRecoveryCleanupFreshnessReadDenialKind::Execution(_) => {
                self.record_freshness_scheduler(true, false, true, false);
            }
            PhysicalRecoveryCleanupFreshnessReadDenialKind::Media => {
                self.record_freshness_scheduler(
                    read.progress().work().is_some(),
                    false,
                    false,
                    read.progress().scheduler().is_some(),
                );
            }
            PhysicalRecoveryCleanupFreshnessReadDenialKind::SchedulerSettlement(_)
            | PhysicalRecoveryCleanupFreshnessReadDenialKind::SignalSettlement(_)
            | PhysicalRecoveryCleanupFreshnessReadDenialKind::InvalidSelector => {
                self.record_freshness_scheduler(true, false, false, true);
            }
            PhysicalRecoveryCleanupFreshnessReadDenialKind::Yieldpoint(_) => {
                self.record_freshness_scheduler(true, false, true, false);
            }
        }
    }

    pub(super) fn record_completed(
        &mut self,
        bytes: u64,
        performed: PerformedRecoveryCleanupRemoval,
        revalidation: RecoveryCleanupArtifactRevalidationProgress,
    ) {
        self.counters.actions_attempted += 1;
        self.counters.actions_completed += 1;
        self.counters.bytes_completed += bytes;
        self.counters.performed_effects += 1;
        self.record_removal_scheduler(true, false, false, true);
        self.record_revalidation(revalidation, None);
        self.performed.push(performed);
    }

    pub(super) fn record_deferral(&mut self, evidence: RecoveryCleanupDeferralEvidence) {
        match &evidence {
            RecoveryCleanupDeferralEvidence::Freshness { failure, .. } => {
                self.record_freshness_failure(failure);
            }
            RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. } => {
                self.counters.actions_attempted += 1;
                self.counters.denied_before_effect += 1;
                match denial.kind() {
                    PhysicalRecoveryCleanupRemovalDenialKind::InvalidCommand => {
                        self.record_removal_scheduler(
                            denial.work().is_some(),
                            false,
                            false,
                            denial.scheduler().is_some(),
                        );
                    }
                    PhysicalRecoveryCleanupRemovalDenialKind::Admission(admission) => {
                        self.record_removal_scheduler(
                            admission.submission_recorded(),
                            admission.scheduler_deferred(),
                            admission.cancellation_recorded(),
                            false,
                        );
                    }
                    PhysicalRecoveryCleanupRemovalDenialKind::Execution(_) => {
                        self.record_removal_scheduler(true, false, true, false);
                    }
                    PhysicalRecoveryCleanupRemovalDenialKind::Media(cause) => {
                        self.record_removal_scheduler(
                            denial.work().is_some(),
                            false,
                            false,
                            denial.scheduler().is_some(),
                        );
                        if let Some(physical) = denial.physical() {
                            let revalidation_denial = match cause {
                                worth_store::physical_runtime::RecoveryCleanupRemovalDenialCause::Revalidation(
                                    denial,
                                ) => Some(*denial),
                                _ => None,
                            };
                            self.record_revalidation(physical.revalidation(), revalidation_denial);
                        }
                    }
                }
            }
            RecoveryCleanupDeferralEvidence::IndeterminateEffect { evidence, .. } => {
                self.counters.actions_attempted += 1;
                self.counters.indeterminate_effects += 1;
                let interrupted = matches!(
                    evidence,
                    PhysicalRecoveryCleanupRemovalIndeterminate::Yieldpoint { .. }
                );
                self.record_removal_scheduler(true, false, interrupted, !interrupted);
                self.record_revalidation(evidence.revalidation(), None);
            }
            RecoveryCleanupDeferralEvidence::PublishedGenerationChanged { .. }
            | RecoveryCleanupDeferralEvidence::EligibilityChanged { .. }
            | RecoveryCleanupDeferralEvidence::Cancelled { .. }
            | RecoveryCleanupDeferralEvidence::CancellationBindingMismatch { .. } => {}
        }
        self.deferrals.push(evidence);
    }

    pub(super) fn record_cancellation(
        &mut self,
        actions: u64,
        bytes: u64,
        evidence: RecoveryCleanupDeferralEvidence,
    ) {
        self.counters.cancellation_requests += 1;
        self.counters.actions_cancelled += actions;
        self.counters.bytes_cancelled += bytes;
        self.deferrals.push(evidence);
    }

    fn record_freshness_scheduler(
        &mut self,
        submitted: bool,
        deferred: bool,
        cancelled: bool,
        settled: bool,
    ) {
        self.counters.freshness_scheduler_submitted += u64::from(submitted);
        self.counters.freshness_scheduler_deferred += u64::from(deferred);
        self.counters.freshness_scheduler_cancelled += u64::from(cancelled);
        self.counters.freshness_scheduler_settled += u64::from(settled);
        self.record_scheduler(submitted, deferred, cancelled, settled);
    }

    fn record_removal_scheduler(
        &mut self,
        submitted: bool,
        deferred: bool,
        cancelled: bool,
        settled: bool,
    ) {
        self.counters.removal_scheduler_submitted += u64::from(submitted);
        self.counters.removal_scheduler_deferred += u64::from(deferred);
        self.counters.removal_scheduler_cancelled += u64::from(cancelled);
        self.counters.removal_scheduler_settled += u64::from(settled);
        self.record_scheduler(submitted, deferred, cancelled, settled);
    }

    fn record_scheduler(
        &mut self,
        submitted: bool,
        deferred: bool,
        cancelled: bool,
        settled: bool,
    ) {
        self.counters.scheduler_commands_submitted += u64::from(submitted);
        self.counters.scheduler_commands_deferred += u64::from(deferred);
        self.counters.scheduler_commands_cancelled += u64::from(cancelled);
        self.counters.scheduler_commands_settled += u64::from(settled);
    }

    fn record_revalidation(
        &mut self,
        progress: RecoveryCleanupArtifactRevalidationProgress,
        denial: Option<RecoveryCleanupArtifactRevalidationDenial>,
    ) {
        self.counters.artifact_revalidation_reads_attempted += progress.reads_attempted();
        self.counters.artifact_revalidation_reads_completed += progress.reads_completed();
        self.counters.artifact_revalidation_bytes_read += progress.bytes_read();
        match denial {
            Some(
                RecoveryCleanupArtifactRevalidationDenial::Read(_)
                | RecoveryCleanupArtifactRevalidationDenial::CheckpointRead(_),
            ) => {
                self.counters.artifact_revalidation_read_failures += 1;
            }
            Some(
                RecoveryCleanupArtifactRevalidationDenial::LengthMismatch { .. }
                | RecoveryCleanupArtifactRevalidationDenial::DigestMismatch { .. }
                | RecoveryCleanupArtifactRevalidationDenial::CheckpointLengthMismatch { .. }
                | RecoveryCleanupArtifactRevalidationDenial::CheckpointDigestMismatch { .. },
            ) => {
                self.counters.artifact_revalidation_mismatches += 1;
            }
            None => {}
        }
    }

    pub(super) fn finish(
        mut self,
        plan: RecoveryCleanupPlan,
        live_media_handles_after_close: u64,
    ) -> RecoveryCleanupPosture {
        self.counters.live_media_handles_after_close = live_media_handles_after_close;
        for disposition in plan.dispositions() {
            match disposition.kind() {
                RecoveryCleanupDispositionKind::Current => self.counters.current += 1,
                RecoveryCleanupDispositionKind::Retained => self.counters.retained += 1,
                RecoveryCleanupDispositionKind::Eligible => {
                    self.counters.eligible_after_cleanup += 1;
                }
                RecoveryCleanupDispositionKind::Deferred(_) => {
                    self.counters.actions_deferred += 1;
                    self.counters.bytes_deferred += disposition.byte_count();
                }
                RecoveryCleanupDispositionKind::QuarantinedOrUnsupported => {
                    self.counters.quarantined_or_unsupported += 1;
                }
                RecoveryCleanupDispositionKind::SafelyRemoved => {
                    self.counters.safely_removed += 1;
                }
            }
        }
        let evidence = RecoveryCleanupEvidence::new(RecoveryCleanupEvidenceParts {
            plan_identity: self.plan_identity,
            published_generation: self.published_generation,
            dispositions: plan.into_dispositions(),
            freshness: self.freshness.into_boxed_slice(),
            performed: self.performed.into_boxed_slice(),
            deferrals: self.deferrals.into_boxed_slice(),
            counters: self.counters,
        });
        RecoveryCleanupPosture::from_evidence(evidence)
    }
}
