use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalSourceSelection, RecoveryPlanCost, RecoveryPlanningCounters,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::{RecoveryCoordination, ResidentSourceSelection};

use super::{PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence};

pub struct PlannedPhysicalRecovery {
    state: PlannedStateBacking,
}

mod state;
use state::PlannedRecoveryState;
pub(crate) use state::PlannedStateBacking;

mod cancellation;
mod staging_input;
pub use cancellation::PhysicalRecoveryStagingCancellation;

impl PlannedPhysicalRecovery {
    pub(crate) fn new(
        backing: PlannedStateBacking,
        authority: AdmittedPlatformAuthority,
        coordination: RecoveryCoordination,
        selection: ResidentSourceSelection,
        custody: crate::progression::PlanningCustody,
        verified_selected_tier_custody: Option<
            worth_store_recovery_physics::VerifiedSelectedTierEpochCustody,
        >,
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
        backing.finish(PlannedRecoveryState {
            authority,
            coordination,
            selection,
            custody,
            verified_selected_tier_custody,
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
        })
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.state.state().authority.media.store_identity()
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
    pub const fn operation_fates(&self) -> &RecoveryOperationFateSet {
        &self.state.state().fates
    }
    pub const fn redo_plan(&self) -> &ImmutablePhysicalRedoPlan {
        &self.state.state().redo
    }
    pub const fn plan_cost(&self) -> RecoveryPlanCost {
        self.state.state().plan_cost
    }
    pub const fn planning_counters(&self) -> RecoveryPlanningCounters {
        self.state.state().planning_counters
    }
    pub const fn root_protocol_counters(
        &self,
    ) -> crate::entry::PhysicalRecoveryRootProtocolCounters {
        self.state.state().root_protocol_counters
    }
    pub const fn staging_layout(&self) -> &RecoveryStagingLayoutPlan {
        &self.state.state().staging
    }
    pub const fn publication_plan(&self) -> &RecoveryPublicationPlan {
        &self.state.state().publication
    }
    pub const fn quiescence_plan(&self) -> RecoveryQuiescencePlan {
        self.state.state().quiescence
    }
    pub const fn selected_sources(&self) -> &PhysicalSourceSelection {
        self.state.state().selection.facts()
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.state.state().integrity_trace.counters().attempted
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.state.state().integrity_trace.observations()
    }

    pub fn cancellation_after_command(
        &self,
        command_ordinal: u64,
    ) -> Option<PhysicalRecoveryStagingCancellation> {
        let index = usize::try_from(command_ordinal).ok()?;
        self.state.state().staging.commands().get(index)?;
        Some(PhysicalRecoveryStagingCancellation::new(
            self.state.state().publication.plan_identity(),
            command_ordinal.checked_add(1)?,
        ))
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_staging_signal_settlement_at(
        &self,
        stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage,
    ) {
        self.state
            .state()
            .coordination
            .fail_signal_settlement_at_for_certification(stage);
    }

    pub fn cancel_before_execution(self) -> PhysicalRecoveryOutcome {
        let PlannedRecoveryState {
            authority,
            coordination,
            integrity,
            root_protocol_denials,
            root_protocol_counters,
            integrity_trace,
            ..
        } = self.state.into_state();
        assert!(coordination.shutdown_is_quiescent());
        let recovery_effects = authority.media.recovery_effect_count();
        let crate::entry::AdmittedPlatformAuthority { media, session, .. } = authority;
        drop(media);
        session.refuse();
        PhysicalRecoveryOutcome::Refused(Box::new(
            crate::entry::PhysicalRecoveryRefusal::new(
                crate::entry::PhysicalRecoveryRefusalKind::CancelledBeforeExecution,
                recovery_effects,
            )
            .with_root_protocol_denials(root_protocol_denials)
            .with_root_protocol_counters(root_protocol_counters)
            .with_integrity_trace(integrity_trace)
            .with_integrity_observations(integrity.into_observations()),
        ))
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
                self.state.state().publication.plan_identity(),
                self.state.state().staging.commands().len() as u64,
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
        let input = staging_input::prepare(self.state, cancellation);
        crate::orchestration::stage_recovery(input)
    }
}
mod basis;
mod resident_memory;
pub(crate) use resident_memory::{PlanningMemoryDenial, PlanningResidentAllowance};

pub(crate) use basis::{
    derive_execution_basis, requires_successor_candidate, verified_historical_release_transition,
    ExecutionBasis, ExecutionBasisDenial, ExecutionBasisState, RecoveryObservedCandidateArtifact,
    RecoveryObservedSuccessorCandidate, RecoverySelectedSegmentPage,
    RecoverySelectedSourceInventory,
};
pub use basis::{
    RecoveryBaseImageAction, RecoveryBaseImagePlan, RecoveryPayloadManifestAction,
    RecoveryPublicationAction, RecoveryPublicationCandidateArtifact,
    RecoveryPublicationExpectation, RecoveryPublicationPlan, RecoveryQuiescencePlan,
    RecoverySegmentRoutingAction, RecoveryStagingAction, RecoveryStagingCommandPlan,
    RecoveryStagingLayoutPlan, RecoveryStagingRedoStep,
};
