use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalSourceSelection, RecoveryPlanCost, RecoveryPlanningCounters,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::RecoveryCoordination;

use super::{PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence};

pub struct PlannedPhysicalRecovery {
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    selection: PhysicalSourceSelection,
    discovery_counters: PhysicalRecoveryDiscoveryCounters,
    root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    integrity: RecoveryIntegrityEvidence,
    freshness: StoreRecoveryBindingFreshnessSample,
    fates: RecoveryOperationFateSet,
    redo: ImmutablePhysicalRedoPlan,
    plan_cost: RecoveryPlanCost,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    staging: RecoveryStagingLayoutPlan,
    publication: RecoveryPublicationPlan,
    quiescence: RecoveryQuiescencePlan,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

mod cancellation;
pub use cancellation::PhysicalRecoveryStagingCancellation;

impl PlannedPhysicalRecovery {
    pub(crate) const fn new(
        authority: AdmittedPlatformAuthority,
        coordination: RecoveryCoordination,
        selection: PhysicalSourceSelection,
        discovery_counters: PhysicalRecoveryDiscoveryCounters,
        root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
        integrity: RecoveryIntegrityEvidence,
        freshness: StoreRecoveryBindingFreshnessSample,
        fates: RecoveryOperationFateSet,
        redo: ImmutablePhysicalRedoPlan,
        plan_cost: RecoveryPlanCost,
        planning_counters: RecoveryPlanningCounters,
        root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
        staging: RecoveryStagingLayoutPlan,
        publication: RecoveryPublicationPlan,
        quiescence: RecoveryQuiescencePlan,
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
            redo,
            plan_cost,
            planning_counters,
            root_protocol_counters,
            staging,
            publication,
            quiescence,
            integrity_trace,
        }
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.authority.media.store_identity()
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
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.fates
    }
    pub const fn redo_plan(&self) -> &ImmutablePhysicalRedoPlan {
        &self.redo
    }
    pub const fn plan_cost(&self) -> RecoveryPlanCost {
        self.plan_cost
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.root_protocol_counters
    }
    pub const fn staging_layout(&self) -> &RecoveryStagingLayoutPlan {
        &self.staging
    }
    pub const fn publication_plan(&self) -> &RecoveryPublicationPlan {
        &self.publication
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.quiescence
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        &self.selection
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }

    pub fn cancellation_after_command(
        &self,
        command_ordinal: u64,
    ) -> Option<PhysicalRecoveryStagingCancellation> {
        let index = usize::try_from(command_ordinal).ok()?;
        self.staging.commands().get(index)?;
        Some(PhysicalRecoveryStagingCancellation::new(
            self.publication.plan_identity(),
            command_ordinal.checked_add(1)?,
        ))
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_staging_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage,
    ) {
        self.coordination
            .fail_signal_settlement_at_for_certification(stage);
    }

    pub fn cancel_before_execution(self) -> PhysicalRecoveryOutcome {
        let Self {
            authority,
            coordination,
            integrity,
            root_protocol_denials,
            root_protocol_counters,
            integrity_trace,
            ..
        } = self;
        assert!(coordination.shutdown_is_quiescent());
        let recovery_effects = authority.media.recovery_effect_count();
        let crate::entry::AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.refuse();
        PhysicalRecoveryOutcome::Refused(
            crate::entry::PhysicalRecoveryRefusal::new(
                crate::entry::PhysicalRecoveryRefusalKind::CancelledBeforeExecution,
                recovery_effects,
            )
            .with_root_protocol_denials(root_protocol_denials)
            .with_root_protocol_counters(root_protocol_counters)
            .with_integrity_trace(integrity_trace)
            .with_integrity_observations(integrity.into_observations()),
        )
    }

    /// Consumes the immutable Phase 4 basis and materializes one closed,
    /// non-current staging generation. No selector or serving root is changed.
    pub fn stage(self) -> Result<super::StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
        self.stage_with_admitted_cancellation(
            crate::orchestration::RecoveryStagingCancellation::None,
        )
    }

    pub fn stage_with_cancellation(
        self,
        cancellation: PhysicalRecoveryStagingCancellation,
    ) -> Result<super::StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
        let admitted = cancellation
            .admit(
                self.publication.plan_identity(),
                self.staging.commands().len() as u64,
            )
            .map_or(
                crate::orchestration::RecoveryStagingCancellation::Invalid,
                crate::orchestration::RecoveryStagingCancellation::AfterSettledCommands,
            );
        self.stage_with_admitted_cancellation(admitted)
    }

    fn stage_with_admitted_cancellation(
        self,
        cancellation: crate::orchestration::RecoveryStagingCancellation,
    ) -> Result<super::StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
        let Self {
            authority,
            coordination,
            selection,
            discovery_counters,
            freshness,
            fates,
            redo: _,
            plan_cost: _,
            planning_counters,
            root_protocol_counters,
            staging,
            publication,
            quiescence,
            root_protocol_denials,
            integrity_trace,
            integrity,
        } = self;
        crate::orchestration::stage_recovery(crate::orchestration::RecoveryStagingInput {
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
            staging,
            publication,
            quiescence,
            cancellation,
            integrity_trace,
        })
    }
}
mod basis;

pub(crate) use basis::{
    derive_execution_basis, requires_successor_candidate, CandidateMaterializationCost,
    ExecutionBasisDenial, RecoveryObservedCandidateArtifact, RecoveryObservedSuccessorCandidate,
    RecoverySelectedSegmentPage, RecoverySelectedSourceInventory,
};
pub use basis::{
    RecoveryBaseImageAction, RecoveryBaseImagePlan, RecoveryPayloadManifestAction,
    RecoveryPublicationAction, RecoveryPublicationCandidateArtifact,
    RecoveryPublicationExpectation, RecoveryPublicationPlan, RecoveryQuiescencePlan,
    RecoverySegmentRoutingAction, RecoveryStagingAction, RecoveryStagingCommandPlan,
    RecoveryStagingLayoutPlan, RecoveryStagingRedoStep,
};
