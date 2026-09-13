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
    pub(crate) state: NamespaceDurableState,
    pub(crate) expectation: RecoveryPublicationExpectation,
    pub(crate) publication_counters: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
}

pub(crate) struct NamespaceDurableState {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) coordination: RecoveryCoordination,
    pub(crate) selection: PhysicalSourceSelection,
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
    pub(crate) const fn new(
        state: NamespaceDurableState,
        expectation: RecoveryPublicationExpectation,
        publication_counters: PhysicalRecoveryPublicationCounters,
        publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
    ) -> Self {
        Self {
            state,
            expectation,
            publication_counters,
            publication_settlement,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.state.authority.media.store_identity()
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
    pub const fn closed_generation(&self) -> ClosedRecoveryStagingGeneration {
        self.state.closed
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.state.fates
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        &self.state.selection
    }
    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.state.discovery_counters
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.state.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.state.integrity.observations().wal()
    }
    pub const fn freshness_sample(&self) -> &StoreRecoveryBindingFreshnessSample {
        &self.state.freshness
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.state.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.state.root_protocol_counters
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.state.base
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.state.quiescence
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
        self.state
            .coordination
            .fail_reopen_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_reopen_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage,
    ) {
        self.state
            .coordination
            .fail_reopen_signal_settlement_at_for_certification(stage);
    }
}
