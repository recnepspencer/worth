use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial,
    PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::RecoveryCoordination;

use super::{
    PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan, RecoveryIntegrityEvidence,
    RecoveryPublicationPlan, RecoveryQuiescencePlan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedRecoveryStagingGeneration {
    generation: u64,
    artifact_count: u64,
    byte_count: u64,
    content_identity: [u8; 32],
}

pub struct StagedPhysicalRecovery {
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
    pub(crate) publication: RecoveryPublicationPlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) closed: ClosedRecoveryStagingGeneration,
    pub(crate) staging_counters: PhysicalRecoveryStagingCounters,
    pub(crate) staging_settlements: PhysicalRecoveryStagingSettlementLedger,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl StagedPhysicalRecovery {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        authority: AdmittedPlatformAuthority,
        coordination: RecoveryCoordination,
        selection: PhysicalSourceSelection,
        discovery_counters: PhysicalRecoveryDiscoveryCounters,
        root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
        integrity: RecoveryIntegrityEvidence,
        freshness: StoreRecoveryBindingFreshnessSample,
        fates: RecoveryOperationFateSet,
        planning_counters: RecoveryPlanningCounters,
        root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
        base: RecoveryBaseImagePlan,
        publication: RecoveryPublicationPlan,
        quiescence: RecoveryQuiescencePlan,
        closed: ClosedRecoveryStagingGeneration,
        staging_counters: PhysicalRecoveryStagingCounters,
        staging_settlements: PhysicalRecoveryStagingSettlementLedger,
        integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        Self {
            authority,
            coordination,
            selection,
            discovery_counters,
            root_protocol_denials,
            integrity,
            freshness,
            fates,
            planning_counters,
            root_protocol_counters,
            base,
            publication,
            quiescence,
            closed,
            staging_counters,
            staging_settlements,
            integrity_trace,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.authority.media.store_identity()
    }
    pub const fn closed_generation(&self) -> ClosedRecoveryStagingGeneration {
        self.closed
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.staging_counters
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.staging_settlements
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.fates
    }
    pub const fn publication_plan(&self) -> &RecoveryPublicationPlan {
        &self.publication
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.base
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        &self.selection
    }
    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.discovery_counters
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.integrity.observations().wal()
    }
    pub const fn freshness_sample(&self) -> &StoreRecoveryBindingFreshnessSample {
        &self.freshness
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.root_protocol_counters
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.quiescence
    }
    pub fn is_quiescent(&self) -> bool {
        self.coordination.is_ready()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_publication_scheduler_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.coordination
            .fail_publication_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_publication_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.coordination
            .fail_publication_signal_settlement_at_for_certification(stage);
    }

    /// Cancels at the closed-staging safe point without publishing any root.
    ///
    /// Staging effects have escaped, so this is a retained `Blocked` posture,
    /// never a counterfeit no-effect refusal.
    pub fn cancel_before_publication(self) -> PhysicalRecoveryOutcome {
        let Self {
            authority,
            coordination,
            integrity,
            discovery_counters,
            root_protocol_denials,
            planning_counters,
            root_protocol_counters,
            staging_counters,
            staging_settlements,
            integrity_trace,
            ..
        } = self;
        assert!(coordination.shutdown_is_quiescent());
        let store = authority.media.store_identity();
        let session_identity = authority.session.identity();
        let recovery_effects = authority.media.recovery_effect_count();
        let crate::entry::AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.block();
        PhysicalRecoveryOutcome::Blocked(crate::entry::PhysicalRecoveryBlock::new(
            crate::entry::PhysicalRecoveryBlockKind::Staging,
            store,
            session_identity,
            crate::entry::PhysicalRecoveryBlockEvidence {
                counters: discovery_counters,
                source_denials: root_protocol_denials,
                integrity_observations: integrity.into_observations(),
                planning_counters: Some(planning_counters),
                root_protocol_counters: Some(root_protocol_counters),
                staging_counters: Some(staging_counters),
                staging_denial: Some(
                    crate::entry::PhysicalRecoveryStagingDenial::CancelledAfterClosedStaging,
                ),
                staging_settlements: Some(staging_settlements),
                integrity_trace,
                ..crate::entry::PhysicalRecoveryBlockEvidence::default()
            },
            recovery_effects,
        ))
    }

    /// Consumes the closed staging generation and executes only the exact
    /// candidate, root-protocol, and namespace actions fixed by Phase 4.
    pub fn publish(
        self,
    ) -> Result<super::NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
        crate::orchestration::publish_recovery(self)
    }
}

impl ClosedRecoveryStagingGeneration {
    pub(crate) const fn new(
        generation: u64,
        artifact_count: u64,
        byte_count: u64,
        content_identity: [u8; 32],
    ) -> Self {
        Self {
            generation,
            artifact_count,
            byte_count,
            content_identity,
        }
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn artifact_count(self) -> u64 {
        self.artifact_count
    }
    pub const fn byte_count(self) -> u64 {
        self.byte_count
    }
    pub const fn content_identity(self) -> [u8; 32] {
        self.content_identity
    }
}
