use super::PhysicalRecoveryCoordination;

impl PhysicalRecoveryCoordination {
    /// Read-only allocation events from the actual carried native pool.
    pub fn certification_residency_allocations(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver {
        self.residency.ports().allocation_events()
    }

    /// Test-only pressure on the carried pool; no media or Serving authority.
    pub fn certification_begin_recovery_allocation(
        &self,
        bytes: std::num::NonZeroU64,
    ) -> Result<
        worth_store_buffer_pool::OperationAllocationGrant,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        self.residency.ports().begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery,
            bytes,
        )
    }

    pub fn certification_fail_signal_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryStagingCommandStage,
    ) {
        self.certification_faults.fail_signal_settlement_at(stage);
    }

    pub fn certification_fail_publication_signal_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.certification_faults
            .fail_publication_signal_settlement_at(stage);
    }

    pub fn certification_fail_reopen_signal_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryFreshReopenStage,
    ) {
        self.certification_faults
            .fail_reopen_signal_settlement_at(stage);
    }

    pub fn certification_fail_reopen_scheduler_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryFreshReopenStage,
    ) {
        self.certification_faults
            .fail_reopen_scheduler_settlement_at(stage);
    }

    pub fn certification_fail_publication_scheduler_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.certification_faults
            .fail_publication_scheduler_settlement_at(stage);
    }

    pub fn certification_shift_cleanup_generation(&self) {
        self.certification_faults.shift_cleanup_generation();
    }

    pub fn certification_fail_cleanup_plan_admission(&self) {
        self.certification_faults.fail_cleanup_plan_admission();
    }

    pub fn certification_fail_cleanup_eligibility_after_read(&self) {
        self.certification_faults
            .fail_cleanup_eligibility_after_read();
    }

    pub fn certification_fail_cleanup_freshness_signal_settlement(&self) {
        self.certification_faults
            .fail_cleanup_freshness_signal_settlement();
    }

    pub fn certification_fail_cleanup_scheduler_settlement_at(
        &self,
        stage: super::super::PhysicalRecoveryCleanupCommandStage,
    ) {
        self.certification_faults
            .fail_cleanup_scheduler_settlement_at(stage);
    }

    pub fn certification_defer_cleanup_background(&self) {
        self.certification_faults.defer_cleanup_background();
    }

    pub fn certification_substitute_cleanup_authorization(&self) {
        self.certification_faults.substitute_cleanup_authorization();
    }

    pub(in crate::physical_runtime) fn take_certification_cleanup_generation_shift(&self) -> bool {
        self.certification_faults.take_cleanup_generation_shift()
    }

    pub fn certification_leak_cleanup_media_handle(&self) {
        self.certification_faults.leak_cleanup_media_handle();
    }

    pub(in crate::physical_runtime) fn take_certification_cleanup_plan_admission_failure(
        &self,
    ) -> bool {
        self.certification_faults
            .take_cleanup_plan_admission_failure()
    }

    pub(in crate::physical_runtime) fn take_certification_cleanup_eligibility_failure(
        &self,
    ) -> bool {
        self.certification_faults.take_cleanup_eligibility_failure()
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_signal_failure(
        &self,
        stage: super::super::settlement::PhysicalRecoverySettlementCertificationStage,
    ) -> bool {
        self.certification_faults.take_signal_failure(stage)
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_reopen_scheduler_failure(
        &self,
        stage: super::super::PhysicalRecoveryFreshReopenStage,
    ) -> bool {
        self.certification_faults
            .take_reopen_scheduler_failure(stage)
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_publication_scheduler_failure(
        &self,
        stage: super::super::PhysicalRecoveryPublicationCommandStage,
    ) -> bool {
        self.certification_faults
            .take_publication_scheduler_failure(stage)
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_cleanup_scheduler_failure(
        &self,
        stage: super::super::PhysicalRecoveryCleanupCommandStage,
    ) -> bool {
        self.certification_faults
            .take_cleanup_scheduler_failure(stage)
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_cleanup_background_deferral(
        &self,
    ) -> bool {
        self.certification_faults.take_cleanup_background_deferral()
    }

    pub(in crate::physical_runtime::recovery_coordination) fn take_certification_cleanup_authorization_substitution(
        &self,
    ) -> bool {
        self.certification_faults
            .take_cleanup_authorization_substitution()
    }

    pub fn take_certification_cleanup_media_handle_leak(&self) -> bool {
        self.certification_faults.take_cleanup_media_handle_leak()
    }
}
