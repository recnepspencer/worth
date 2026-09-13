use worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;

use super::{NamespaceDurableState, RecoveryPublicationExpectation};

pub struct ReopenedPhysicalRecovery {
    pub(crate) state: NamespaceDurableState,
    pub(crate) expectation: RecoveryPublicationExpectation,
    pub(crate) publication_counters: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
    pub(crate) reopened: Option<CompletedPhysicalRecoveryFreshReopen>,
    pub(crate) reopen_counters: PhysicalRecoveryReopenCounters,
}

impl ReopenedPhysicalRecovery {
    pub(crate) const fn new(
        state: NamespaceDurableState,
        expectation: RecoveryPublicationExpectation,
        publication_counters: PhysicalRecoveryPublicationCounters,
        publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
        reopened: CompletedPhysicalRecoveryFreshReopen,
        reopen_counters: PhysicalRecoveryReopenCounters,
    ) -> Self {
        Self {
            state,
            expectation,
            publication_counters,
            publication_settlement,
            reopened: Some(reopened),
            reopen_counters,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.state.authority.media.store_identity()
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        self.reopened
            .as_ref()
            .expect("fresh reopen remains present before cleanup entry")
            .root()
    }
    pub const fn reopen_counters(&self) -> PhysicalRecoveryReopenCounters {
        self.reopen_counters
    }
    pub const fn fresh_reopen(&self) -> &CompletedPhysicalRecoveryFreshReopen {
        self.reopened
            .as_ref()
            .expect("fresh reopen remains present before cleanup entry")
    }
    pub const fn publication_expectation(&self) -> &RecoveryPublicationExpectation {
        &self.expectation
    }
    pub const fn publication_counters(&self) -> PhysicalRecoveryPublicationCounters {
        self.publication_counters
    }
    pub const fn publication_settlement(&self) -> &PhysicalRecoveryPublicationSettlementLedger {
        &self.publication_settlement
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.state.fates
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        &self.state.selection
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.state.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.state.integrity.observations().wal()
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.state.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.state.root_protocol_counters
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.state.staging_counters
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.state.staging_settlements
    }
    pub fn is_quiescent(&self) -> bool {
        self.state.coordination.is_ready()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.state.integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.state.integrity_trace.observations()
    }

    pub(crate) fn take_fresh_reopen(&mut self) -> CompletedPhysicalRecoveryFreshReopen {
        self.reopened
            .take()
            .expect("cleanup entry consumes fresh reopen exactly once")
    }

    /// Consumes the recovery-only runtime and returns the narrow Store-owned
    /// recovered-runtime handoff after publication and fresh reopen close.
    pub fn finish(self) -> crate::entry::PhysicalRecoveryOutcome {
        crate::cleanup::execute(self, None)
    }

    /// Issues a consuming request to defer cleanup before its first optional
    /// removal. Recovery success and the reopened root remain unchanged.
    pub fn cleanup_cancellation_before_first(
        &self,
    ) -> Option<crate::cleanup::PhysicalRecoveryCleanupCancellation> {
        crate::cleanup::before_first(self)
    }

    /// Issues a consuming request to defer cleanup after one exact zero-based
    /// removal ordinal has settled and before the next candidate starts.
    pub fn cleanup_cancellation_after_removal(
        &self,
        action_ordinal: u64,
    ) -> Option<crate::cleanup::PhysicalRecoveryCleanupCancellation> {
        crate::cleanup::after_action(self, action_ordinal)
    }

    pub fn finish_with_cleanup_cancellation(
        self,
        cancellation: crate::cleanup::PhysicalRecoveryCleanupCancellation,
    ) -> crate::entry::PhysicalRecoveryOutcome {
        crate::cleanup::execute(self, Some(cancellation))
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_shift_cleanup_generation(&self) {
        self.state
            .coordination
            .shift_cleanup_generation_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_plan_admission(&self) {
        self.state
            .coordination
            .fail_cleanup_plan_admission_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_eligibility_after_read(&self) {
        self.state
            .coordination
            .fail_cleanup_eligibility_after_read_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_freshness_signal_settlement(&self) {
        self.state
            .coordination
            .fail_cleanup_freshness_signal_settlement_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_scheduler_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryCleanupCommandStage,
    ) {
        self.state
            .coordination
            .fail_cleanup_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_defer_cleanup_background(&self) {
        self.state
            .coordination
            .defer_cleanup_background_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_substitute_cleanup_authorization(&self) {
        self.state
            .coordination
            .certification_substitute_cleanup_authorization();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_leak_cleanup_media_handle(&self) {
        self.state
            .coordination
            .certification_leak_cleanup_media_handle();
    }
}
