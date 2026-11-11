use crate::orchestration::ResidentSourceSelection;
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::RecoveryPlanningCounters;

use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger, RecoverySession,
};
use crate::progression::{
    ClosedRecoveryStagingGeneration, PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan,
    RecoveryPublicationExpectation, RecoveryQuiescencePlan,
};

use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate};
use crate::handoff::{
    RecoveredPhysicalRuntimeHandoff, RecoveredPhysicalRuntimeHandoffEvidence,
    RecoveryCleanupPosture, RecoveryOperationFateSet,
};
use worth_store::physical_runtime::{
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};

pub(crate) struct PendingCompletion {
    pub(crate) store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    pub(crate) session_identity: crate::entry::PhysicalRecoverySessionIdentity,
    pub(crate) recovery_effects: u64,
    /// Measured live Runtime and cleanup backing at the Store construction
    /// boundary, before any independent Store rejoin allocation.
    pub(crate) store_rejoin_retained_bytes: Option<u64>,
    pub(crate) session: RecoverySession,
    pub(crate) selection: ResidentSourceSelection,
    pub(crate) discovery: PhysicalRecoveryDiscoveryCounters,
    pub(crate) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(crate) integrity_observations: crate::entry::PhysicalRecoveryIntegrityObservations,
    pub(crate) freshness: StoreRecoveryBindingFreshnessSample,
    pub(crate) fates: RecoveryOperationFateSet,
    pub(crate) planning: RecoveryPlanningCounters,
    pub(crate) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(crate) base: RecoveryBaseImagePlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) closed: ClosedRecoveryStagingGeneration,
    pub(crate) staging: PhysicalRecoveryStagingCounters,
    pub(crate) staging_settlements: PhysicalRecoveryStagingSettlementLedger,
    pub(crate) publication_expectation: RecoveryPublicationExpectation,
    pub(crate) publication: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
    pub(crate) reopen: PhysicalRecoveryReopenCounters,
    pub(crate) cleanup: RecoveryCleanupPosture,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl PendingCompletion {
    pub(super) fn recovered(self, core: RecoveredPhysicalRuntimeCore) -> PhysicalRecoveryOutcome {
        PhysicalRecoveryOutcome::Recovered(Box::new(RecoveredPhysicalRuntimeHandoff::new(
            core,
            RecoveredPhysicalRuntimeHandoffEvidence {
                store_rejoin_retained_bytes: self.store_rejoin_retained_bytes,
                session: self.session.recovered(),
                selection: self.selection,
                discovery: self.discovery,
                root_protocol_denials: self.root_protocol_denials,
                integrity_observations: self.integrity_observations,
                freshness: self.freshness,
                fates: self.fates,
                planning: self.planning,
                root_protocol_counters: self.root_protocol_counters,
                base: self.base,
                quiescence: self.quiescence,
                closed: self.closed,
                staging: self.staging,
                staging_settlements: self.staging_settlements,
                publication_expectation: self.publication_expectation,
                publication: self.publication,
                publication_settlement: self.publication_settlement,
                reopen: self.reopen,
                cleanup: self.cleanup,
                integrity_trace: self.integrity_trace,
            },
        )))
    }
    pub(super) fn refused(
        self,
        denial: RecoveredPhysicalRuntimeConstructionDenial,
    ) -> PhysicalRecoveryOutcome {
        self.session.publication_indeterminate();
        PhysicalRecoveryOutcome::PublicationIndeterminate(Box::new(
            PhysicalRecoveryPublicationIndeterminate::new(
                self.store,
                self.session_identity,
                self.publication,
                self.publication_settlement,
                self.root_protocol_denials,
                self.root_protocol_counters,
                self.recovery_effects,
            )
            .with_integrity_trace(self.integrity_trace)
            .with_integrity_observations(self.integrity_observations)
            .with_handoff_failure(denial)
            .with_cleanup_posture(self.cleanup),
        ))
    }
}
#[cfg(test)]
#[path = "completion_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
