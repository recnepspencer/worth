use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryPublicationCounters,
    PhysicalRecoveryPublicationSettlementLedger, PhysicalRecoverySourceDenial,
    PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::RecoveryCoordination;

use super::{
    ClosedRecoveryStagingGeneration, PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan,
    RecoveryIntegrityEvidence, RecoveryPublicationExpectation, RecoveryQuiescencePlan,
};

pub struct NamespaceDurablePhysicalRecovery {
    backing: NamespaceDurableBacking,
}
mod storage;
pub(crate) use storage::{NamespaceDurableBacking, NamespaceDurablePayload};

pub(crate) struct NamespaceDurableState<C = crate::progression::PlanningCustody> {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) coordination: RecoveryCoordination,
    pub(crate) selection: crate::orchestration::ResidentSourceSelection,
    pub(crate) custody: C,
    pub(crate) verified_selected_tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(crate) discovery_counters: PhysicalRecoveryDiscoveryCounters,
    pub(crate) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(crate) integrity: RecoveryIntegrityEvidence,
    pub(crate) freshness: StoreRecoveryBindingFreshnessSample,
    pub(crate) fates: RecoveryOperationFateSet,
    pub(crate) planning_counters: RecoveryPlanningCounters,
    pub(crate) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(crate) base: RecoveryBaseImagePlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) closed: ClosedRecoveryStagingGeneration,
    pub(crate) staging_counters: PhysicalRecoveryStagingCounters,
    pub(crate) staging_settlements: PhysicalRecoveryStagingSettlementLedger,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl NamespaceDurablePhysicalRecovery {
    pub(crate) const fn payload(&self) -> &NamespaceDurablePayload {
        self.backing.payload()
    }
    pub(crate) fn into_payload(self) -> NamespaceDurablePayload {
        self.backing.into_payload()
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.backing
            .payload()
            .state
            .authority
            .media
            .store_identity()
    }
    pub const fn publication_expectation(&self) -> &RecoveryPublicationExpectation {
        &self.backing.payload().expectation
    }
    pub const fn publication_counters(&self) -> PhysicalRecoveryPublicationCounters {
        self.backing.payload().publication_counters
    }
    pub const fn publication_settlement(&self) -> &PhysicalRecoveryPublicationSettlementLedger {
        &self.backing.payload().publication_settlement
    }
    pub const fn closed_generation(&self) -> ClosedRecoveryStagingGeneration {
        self.backing.payload().state.closed
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.backing.payload().state.fates
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        self.backing.payload().state.selection.facts()
    }
    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.backing.payload().state.discovery_counters
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.backing.payload().state.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.backing.payload().state.integrity.observations().wal()
    }
    pub const fn freshness_sample(&self) -> &StoreRecoveryBindingFreshnessSample {
        &self.backing.payload().state.freshness
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.backing.payload().state.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.backing.payload().state.root_protocol_counters
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.backing.payload().state.base
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.backing.payload().state.quiescence
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.backing.payload().state.staging_counters
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.backing.payload().state.staging_settlements
    }
    pub fn is_quiescent(&self) -> bool {
        self.backing.payload().state.coordination.is_ready()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.backing
            .payload()
            .state
            .integrity_trace
            .counters()
            .attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.backing.payload().state.integrity_trace.observations()
    }

    /// Reopens the namespace-durable selector and root through scheduled C4
    /// reads and proves their exact equality to the sealed publication plan.
    pub fn reopen(
        self,
    ) -> Result<super::ReopenedPhysicalRecovery, crate::entry::PhysicalRecoveryOutcome> {
        crate::orchestration::reopen_recovery(self)
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_reopen_scheduler_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage,
    ) {
        self.backing
            .payload()
            .state
            .coordination
            .fail_reopen_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_reopen_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage,
    ) {
        self.backing
            .payload()
            .state
            .coordination
            .fail_reopen_signal_settlement_at_for_certification(stage);
    }
}
