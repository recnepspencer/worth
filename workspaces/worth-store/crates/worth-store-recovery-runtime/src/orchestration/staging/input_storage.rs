//! Pre-admitted actual staging input; no empty executable owner escapes.
use super::RecoveryStagingCancellation;
use crate::entry::{AdmittedPlatformAuthority, PhysicalRecoverySourceDenial};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::{RecoveryCoordination, ResidentSourceSelection};
use crate::progression::{
    PhysicalRecoveryDiscoveryCounters, PlanningMemoryDenial, PlanningResidentAllowance,
    RecoveryIntegrityEvidence, RecoveryPublicationPlan, RecoveryQuiescencePlan,
    RecoveryStagingLayoutPlan, StagedStateBacking,
};
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::RecoveryPlanningCounters;

pub(crate) struct RecoveryStagingInputState {
    pub(crate) reopened_backing: crate::progression::ReopenedBacking,
    pub(crate) publication_backing: crate::orchestration::PublicationStateBacking,
    pub(crate) namespace_backing: crate::progression::NamespaceDurableBacking,
    pub(crate) staged_backing: StagedStateBacking,
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) coordination: RecoveryCoordination,
    pub(crate) selection: ResidentSourceSelection,
    pub(crate) custody: crate::progression::PlanningCustody,
    pub(crate) verified_selected_tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(crate) discovery_counters: PhysicalRecoveryDiscoveryCounters,
    pub(crate) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(crate) integrity: RecoveryIntegrityEvidence,
    pub(crate) freshness: StoreRecoveryBindingFreshnessSample,
    pub(crate) fates: RecoveryOperationFateSet,
    pub(crate) planning_counters: RecoveryPlanningCounters,
    pub(crate) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(crate) staging: RecoveryStagingLayoutPlan,
    pub(crate) publication: RecoveryPublicationPlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) cancellation: RecoveryStagingCancellation,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

pub(crate) struct RecoveryStagingInputBacking(Vec<RecoveryStagingInputState>);
pub(crate) struct RecoveryStagingInput {
    backing: RecoveryStagingInputBacking,
}

impl RecoveryStagingInputBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self(allowance.reserve::<RecoveryStagingInputState>(1)?))
    }

    #[cfg(test)]
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.0)
    }

    pub(crate) fn finish(mut self, state: RecoveryStagingInputState) -> RecoveryStagingInput {
        self.0.push(state);
        RecoveryStagingInput { backing: self }
    }
}

impl RecoveryStagingInput {
    pub(super) fn state(&self) -> &RecoveryStagingInputState {
        &self.backing.0[0]
    }
    pub(super) fn integrity_trace_mut(
        &mut self,
    ) -> &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace {
        &mut self.backing.0[0].integrity_trace
    }
    pub(super) fn into_state(mut self) -> RecoveryStagingInputState {
        self.backing
            .0
            .pop()
            .expect("staging input contains its admitted state")
    }
}
