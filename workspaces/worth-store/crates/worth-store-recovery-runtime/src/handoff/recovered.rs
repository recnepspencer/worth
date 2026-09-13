use worth_store::physical_runtime::RecoveredPhysicalRuntimeCore;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::progression::{
    ClosedRecoveryStagingGeneration, PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan,
    RecoveryPublicationExpectation, RecoveryQuiescencePlan,
};

use super::{
    RecoveredPhysicalRuntimeHandoffEvidence, RecoveryCleanupPosture, RecoveryOperationFateSet,
};

pub struct RecoveredPhysicalRuntimeHandoff {
    core: RecoveredPhysicalRuntimeCore,
    evidence: RecoveredPhysicalRuntimeHandoffEvidence,
}

impl RecoveredPhysicalRuntimeHandoff {
    pub(crate) const fn new(
        core: RecoveredPhysicalRuntimeCore,
        evidence: RecoveredPhysicalRuntimeHandoffEvidence,
    ) -> Self {
        Self { core, evidence }
    }

    pub const fn core(&self) -> &RecoveredPhysicalRuntimeCore {
        &self.core
    }
    pub fn recovered_session_identity(&self) -> crate::entry::PhysicalRecoverySessionIdentity {
        self.evidence.session.identity()
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.evidence.fates
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        &self.evidence.selection
    }
    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.evidence.discovery
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.evidence.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.evidence.integrity_observations.wal()
    }
    pub const fn freshness_sample(
        &self,
    ) -> &worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample {
        &self.evidence.freshness
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.evidence.base
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.evidence.quiescence
    }
    pub const fn closed_generation(&self) -> ClosedRecoveryStagingGeneration {
        self.evidence.closed
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.evidence.planning
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.evidence.root_protocol_counters
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.evidence.staging
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.evidence.staging_settlements
    }
    pub const fn publication_expectation(&self) -> &RecoveryPublicationExpectation {
        &self.evidence.publication_expectation
    }
    pub const fn publication_counters(&self) -> PhysicalRecoveryPublicationCounters {
        self.evidence.publication
    }
    pub const fn publication_settlement(&self) -> &PhysicalRecoveryPublicationSettlementLedger {
        &self.evidence.publication_settlement
    }
    pub const fn reopen_counters(&self) -> PhysicalRecoveryReopenCounters {
        self.evidence.reopen
    }
    pub const fn cleanup_posture(&self) -> &RecoveryCleanupPosture {
        &self.evidence.cleanup
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.evidence.integrity_trace.counters().attempted
    }

    /// Read-only ingress evidence; constructing a copy grants no recovery authority.
    pub const fn integrity_counters(&self) -> crate::PhysicalRecoveryIntegrityCounters {
        self.evidence.integrity_trace.counters()
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.evidence.integrity_trace.observations()
    }
}

impl std::fmt::Debug for RecoveredPhysicalRuntimeHandoff {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecoveredPhysicalRuntimeHandoff")
            .field("store", &self.core.store_identity())
            .field("runtime", &self.core.runtime_identity())
            .field("root_generation", &self.core.root().generation())
            .field("reopen", &self.evidence.reopen)
            .finish()
    }
}
