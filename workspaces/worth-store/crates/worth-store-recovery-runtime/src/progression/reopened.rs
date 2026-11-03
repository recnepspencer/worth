use worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;

use super::RecoveryPublicationExpectation;

pub struct ReopenedPhysicalRecovery {
    backing: ReopenedBacking,
}
mod storage;
pub(crate) use storage::{ReopenedBacking, ReopenedPayload};

impl ReopenedPhysicalRecovery {
    pub(crate) const fn payload(&self) -> &ReopenedPayload {
        self.backing.payload()
    }
    pub(crate) fn into_payload(self) -> ReopenedPayload {
        self.backing.into_payload()
    }
    pub(crate) fn backing_bytes(&self) -> Option<u64> {
        self.backing.backing_bytes().ok()
    }
    pub(crate) fn release_admitted_wal(&mut self) {
        self.backing.release_admitted_wal();
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.payload().state.authority.media.store_identity()
    }
    pub const fn recovered_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        self.payload()
            .reopened
            .as_ref()
            .expect("fresh reopen remains present before cleanup entry")
            .root()
    }
    pub const fn reopen_counters(&self) -> PhysicalRecoveryReopenCounters {
        self.payload().reopen_counters
    }
    pub const fn fresh_reopen(&self) -> &CompletedPhysicalRecoveryFreshReopen {
        self.payload()
            .reopened
            .as_ref()
            .expect("fresh reopen remains present before cleanup entry")
    }
    pub const fn publication_expectation(&self) -> &RecoveryPublicationExpectation {
        &self.payload().expectation
    }
    pub const fn publication_counters(&self) -> PhysicalRecoveryPublicationCounters {
        self.payload().publication_counters
    }
    pub const fn publication_settlement(&self) -> &PhysicalRecoveryPublicationSettlementLedger {
        &self.payload().publication_settlement
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.payload().state.fates
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        self.payload().state.selection.facts()
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.payload().state.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.payload().state.integrity.observations().wal()
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.payload().state.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.payload().state.root_protocol_counters
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.payload().state.staging_counters
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.payload().state.staging_settlements
    }
    pub fn is_quiescent(&self) -> bool {
        self.payload().state.coordination.is_ready()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.payload().state.integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.payload().state.integrity_trace.observations()
    }

    pub(crate) fn take_fresh_reopen(&mut self) -> CompletedPhysicalRecoveryFreshReopen {
        self.backing.take_fresh_reopen()
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

    /// Applies `change` after Store's checkpoint residue gate and before
    /// cleanup's first byte-exact revalidation, then finishes with the
    /// change left in place.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_finish_with_change_before_cleanup_revalidation(
        self,
        change: impl FnOnce() + 'static,
    ) -> crate::entry::PhysicalRecoveryOutcome {
        let _hook = crate::certification::CleanupRevalidationHook::install(change);
        self.finish()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_shift_cleanup_generation(&self) {
        self.payload()
            .state
            .coordination
            .shift_cleanup_generation_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_plan_admission(&self) {
        self.payload()
            .state
            .coordination
            .fail_cleanup_plan_admission_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_eligibility_after_read(&self) {
        self.payload()
            .state
            .coordination
            .fail_cleanup_eligibility_after_read_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_freshness_signal_settlement(&self) {
        self.payload()
            .state
            .coordination
            .fail_cleanup_freshness_signal_settlement_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_cleanup_scheduler_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryCleanupCommandStage,
    ) {
        self.payload()
            .state
            .coordination
            .fail_cleanup_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_defer_cleanup_background(&self) {
        self.payload()
            .state
            .coordination
            .defer_cleanup_background_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_substitute_cleanup_authorization(&self) {
        self.payload()
            .state
            .coordination
            .certification_substitute_cleanup_authorization();
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_leak_cleanup_media_handle(&self) {
        self.payload()
            .state
            .coordination
            .certification_leak_cleanup_media_handle();
    }
}
