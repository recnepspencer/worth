use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;

use super::{
    PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan, RecoveryPublicationPlan,
    RecoveryQuiescencePlan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedRecoveryStagingGeneration {
    generation: u64,
    artifact_count: u64,
    byte_count: u64,
    content_identity: [u8; 32],
}

pub struct StagedPhysicalRecovery {
    state: StagedStateBacking,
}

mod state;
pub(crate) use state::{StagedRecoveryState, StagedStateBacking};

impl StagedPhysicalRecovery {
    pub(crate) fn into_state(self) -> StagedRecoveryState {
        self.state.into_state()
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.state.state().authority.media.store_identity()
    }
    pub const fn closed_generation(&self) -> ClosedRecoveryStagingGeneration {
        self.state.state().closed
    }
    pub const fn staging_counters(&self) -> PhysicalRecoveryStagingCounters {
        self.state.state().staging_counters
    }
    pub const fn staging_settlements(&self) -> &PhysicalRecoveryStagingSettlementLedger {
        &self.state.state().staging_settlements
    }
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.state.state().fates
    }
    pub const fn publication_plan(&self) -> &RecoveryPublicationPlan {
        &self.state.state().publication
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.state.state().base
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        self.state.state().selection.facts()
    }
    pub const fn discovery_counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.state.state().discovery_counters
    }
    pub fn root_protocol_denials(&self) -> &[PhysicalRecoverySourceDenial] {
        &self.state.state().root_protocol_denials
    }
    pub fn wal_integrity_observations(
        &self,
    ) -> &[crate::entry::PhysicalRecoveryWalIntegrityObservation] {
        self.state.state().integrity.observations().wal()
    }
    pub const fn freshness_sample(&self) -> &StoreRecoveryBindingFreshnessSample {
        &self.state.state().freshness
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.state.state().planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.state.state().root_protocol_counters
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.state.state().quiescence
    }
    pub fn is_quiescent(&self) -> bool {
        self.state.state().coordination.is_ready()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.state.state().integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.state.state().integrity_trace.observations()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_publication_scheduler_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.state
            .state()
            .coordination
            .fail_publication_scheduler_settlement_at_for_certification(stage);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_publication_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage,
    ) {
        self.state
            .state()
            .coordination
            .fail_publication_signal_settlement_at_for_certification(stage);
    }

    /// Cancels at the closed-staging safe point without publishing any root.
    ///
    /// Staging effects have escaped, so this is a retained `Blocked` posture,
    /// never a counterfeit no-effect refusal.
    pub fn cancel_before_publication(self) -> PhysicalRecoveryOutcome {
        let StagedRecoveryState {
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
        } = self.into_state();
        assert!(coordination.shutdown_is_quiescent());
        let store = authority.media.store_identity();
        let session_identity = authority.session.identity();
        let recovery_effects = authority.media.recovery_effect_count();
        let crate::entry::AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.block();
        PhysicalRecoveryOutcome::Blocked(Box::new(crate::entry::PhysicalRecoveryBlock::new(
            crate::entry::PhysicalRecoveryBlockCause::Damage(
                crate::entry::PhysicalRecoveryBlockKind::Staging,
            ),
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
        )))
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
