use std::sync::atomic::{AtomicU8, Ordering};

use super::super::{
    PhysicalRecoveryCleanupCommandStage, PhysicalRecoveryFreshReopenStage,
    PhysicalRecoveryPublicationCommandStage, PhysicalRecoveryStagingCommandStage,
};

pub(super) struct RecoveryCoordinationCertificationFaults {
    staging_signal_failure_stage: AtomicU8,
    publication_signal_failure_stage: AtomicU8,
    reopen_signal_failure_stage: AtomicU8,
    publication_scheduler_failure_stage: AtomicU8,
    reopen_scheduler_failure_stage: AtomicU8,
    cleanup_generation_shift: AtomicU8,
    cleanup_plan_admission_failure: AtomicU8,
    cleanup_eligibility_failure: AtomicU8,
    cleanup_signal_failure_stage: AtomicU8,
    cleanup_scheduler_failure_stage: AtomicU8,
    cleanup_background_deferral: AtomicU8,
    cleanup_authorization_substitution: AtomicU8,
    cleanup_media_handle_leak: AtomicU8,
}

impl RecoveryCoordinationCertificationFaults {
    pub(super) const fn new() -> Self {
        Self {
            staging_signal_failure_stage: AtomicU8::new(0),
            publication_signal_failure_stage: AtomicU8::new(0),
            reopen_signal_failure_stage: AtomicU8::new(0),
            publication_scheduler_failure_stage: AtomicU8::new(0),
            reopen_scheduler_failure_stage: AtomicU8::new(0),
            cleanup_generation_shift: AtomicU8::new(0),
            cleanup_plan_admission_failure: AtomicU8::new(0),
            cleanup_eligibility_failure: AtomicU8::new(0),
            cleanup_signal_failure_stage: AtomicU8::new(0),
            cleanup_scheduler_failure_stage: AtomicU8::new(0),
            cleanup_background_deferral: AtomicU8::new(0),
            cleanup_authorization_substitution: AtomicU8::new(0),
            cleanup_media_handle_leak: AtomicU8::new(0),
        }
    }

    pub(super) fn fail_signal_settlement_at(&self, stage: PhysicalRecoveryStagingCommandStage) {
        self.staging_signal_failure_stage
            .store(staging_stage(stage), Ordering::Release);
    }

    pub(super) fn fail_publication_signal_settlement_at(
        &self,
        stage: PhysicalRecoveryPublicationCommandStage,
    ) {
        self.publication_signal_failure_stage
            .store(publication_stage(stage), Ordering::Release);
    }

    pub(super) fn fail_reopen_signal_settlement_at(&self, stage: PhysicalRecoveryFreshReopenStage) {
        self.reopen_signal_failure_stage
            .store(reopen_stage(stage), Ordering::Release);
    }

    pub(super) fn fail_reopen_scheduler_settlement_at(
        &self,
        stage: PhysicalRecoveryFreshReopenStage,
    ) {
        self.reopen_scheduler_failure_stage
            .store(reopen_stage(stage), Ordering::Release);
    }

    pub(super) fn fail_publication_scheduler_settlement_at(
        &self,
        stage: PhysicalRecoveryPublicationCommandStage,
    ) {
        self.publication_scheduler_failure_stage
            .store(publication_stage(stage), Ordering::Release);
    }

    pub(super) fn fail_cleanup_freshness_signal_settlement(&self) {
        self.cleanup_signal_failure_stage.store(
            cleanup_stage(PhysicalRecoveryCleanupCommandStage::FreshnessRead),
            Ordering::Release,
        );
    }

    pub(super) fn fail_cleanup_scheduler_settlement_at(
        &self,
        stage: PhysicalRecoveryCleanupCommandStage,
    ) {
        self.cleanup_scheduler_failure_stage
            .store(cleanup_stage(stage), Ordering::Release);
    }

    pub(super) fn defer_cleanup_background(&self) {
        self.cleanup_background_deferral.store(1, Ordering::Release);
    }

    pub(super) fn substitute_cleanup_authorization(&self) {
        self.cleanup_authorization_substitution
            .store(1, Ordering::Release);
    }

    pub(super) fn take_signal_failure(
        &self,
        stage: super::super::settlement::PhysicalRecoverySettlementCertificationStage,
    ) -> bool {
        match stage {
            super::super::settlement::PhysicalRecoverySettlementCertificationStage::Staging(
                stage,
            ) => take(&self.staging_signal_failure_stage, staging_stage(stage)),
            super::super::settlement::PhysicalRecoverySettlementCertificationStage::Publication(
                stage,
            ) => take(
                &self.publication_signal_failure_stage,
                publication_stage(stage),
            ),
            super::super::settlement::PhysicalRecoverySettlementCertificationStage::FreshReopen(
                stage,
            ) => take(&self.reopen_signal_failure_stage, reopen_stage(stage)),
            super::super::settlement::PhysicalRecoverySettlementCertificationStage::Cleanup(
                stage,
            ) => {
                stage == PhysicalRecoveryCleanupCommandStage::FreshnessRead
                    && take(&self.cleanup_signal_failure_stage, cleanup_stage(stage))
            }
        }
    }

    pub(super) fn take_reopen_scheduler_failure(
        &self,
        stage: PhysicalRecoveryFreshReopenStage,
    ) -> bool {
        take(&self.reopen_scheduler_failure_stage, reopen_stage(stage))
    }

    pub(super) fn take_publication_scheduler_failure(
        &self,
        stage: PhysicalRecoveryPublicationCommandStage,
    ) -> bool {
        take(
            &self.publication_scheduler_failure_stage,
            publication_stage(stage),
        )
    }

    pub(super) fn shift_cleanup_generation(&self) {
        self.cleanup_generation_shift.store(1, Ordering::Release);
    }

    pub(super) fn leak_cleanup_media_handle(&self) {
        self.cleanup_media_handle_leak.store(1, Ordering::Release);
    }

    pub(super) fn fail_cleanup_plan_admission(&self) {
        self.cleanup_plan_admission_failure
            .store(1, Ordering::Release);
    }

    pub(super) fn fail_cleanup_eligibility_after_read(&self) {
        self.cleanup_eligibility_failure.store(1, Ordering::Release);
    }

    pub(super) fn take_cleanup_generation_shift(&self) -> bool {
        take(&self.cleanup_generation_shift, 1)
    }

    pub(super) fn take_cleanup_plan_admission_failure(&self) -> bool {
        take(&self.cleanup_plan_admission_failure, 1)
    }

    pub(super) fn take_cleanup_eligibility_failure(&self) -> bool {
        take(&self.cleanup_eligibility_failure, 1)
    }

    pub(super) fn take_cleanup_scheduler_failure(
        &self,
        stage: PhysicalRecoveryCleanupCommandStage,
    ) -> bool {
        take(&self.cleanup_scheduler_failure_stage, cleanup_stage(stage))
    }

    pub(super) fn take_cleanup_background_deferral(&self) -> bool {
        take(&self.cleanup_background_deferral, 1)
    }

    pub(super) fn take_cleanup_authorization_substitution(&self) -> bool {
        take(&self.cleanup_authorization_substitution, 1)
    }

    pub(super) fn take_cleanup_media_handle_leak(&self) -> bool {
        take(&self.cleanup_media_handle_leak, 1)
    }
}

fn take(stage: &AtomicU8, expected: u8) -> bool {
    stage
        .compare_exchange(expected, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

const fn staging_stage(stage: PhysicalRecoveryStagingCommandStage) -> u8 {
    match stage {
        PhysicalRecoveryStagingCommandStage::Materialization => 1,
        PhysicalRecoveryStagingCommandStage::Synchronization => 2,
    }
}

const fn cleanup_stage(stage: PhysicalRecoveryCleanupCommandStage) -> u8 {
    match stage {
        PhysicalRecoveryCleanupCommandStage::FreshnessRead => 1,
        PhysicalRecoveryCleanupCommandStage::Removal => 2,
    }
}

const fn reopen_stage(stage: PhysicalRecoveryFreshReopenStage) -> u8 {
    match stage {
        PhysicalRecoveryFreshReopenStage::CurrentSelector => 3,
        PhysicalRecoveryFreshReopenStage::RootManifest => 4,
        PhysicalRecoveryFreshReopenStage::ExactBinding => 5,
    }
}

const fn publication_stage(stage: PhysicalRecoveryPublicationCommandStage) -> u8 {
    match stage {
        PhysicalRecoveryPublicationCommandStage::CandidateMaterialization => 1,
        PhysicalRecoveryPublicationCommandStage::CandidateSynchronization => 2,
        PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement => 3,
        PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization => 4,
        PhysicalRecoveryPublicationCommandStage::StagingResidueRemoval => 5,
    }
}
