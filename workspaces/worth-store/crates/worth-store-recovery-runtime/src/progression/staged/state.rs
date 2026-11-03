use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::RecoveryPlanningCounters;

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::{RecoveryCoordination, ResidentSourceSelection};

use super::{
    PhysicalRecoveryDiscoveryCounters, RecoveryBaseImagePlan, RecoveryPublicationPlan,
    RecoveryQuiescencePlan,
};

use super::ClosedRecoveryStagingGeneration;
use crate::progression::{
    PlanningMemoryDenial, PlanningResidentAllowance, RecoveryIntegrityEvidence,
};

pub(crate) struct StagedRecoveryState {
    pub(crate) publication_backing: crate::orchestration::PublicationStateBacking,
    pub(crate) namespace_backing: crate::progression::NamespaceDurableBacking,
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
    pub(crate) base: RecoveryBaseImagePlan,
    pub(crate) publication: RecoveryPublicationPlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) closed: ClosedRecoveryStagingGeneration,
    pub(crate) staging_counters: PhysicalRecoveryStagingCounters,
    pub(crate) staging_settlements: PhysicalRecoveryStagingSettlementLedger,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

/// Storage admitted before the first staging effect; no empty phase escapes.
pub(crate) struct StagedStateBacking(Vec<StagedRecoveryState>);

impl StagedStateBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self(allowance.reserve::<StagedRecoveryState>(1)?))
    }

    #[cfg(test)]
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.0)
    }

    pub(crate) fn finish(mut self, state: StagedRecoveryState) -> super::StagedPhysicalRecovery {
        self.0.push(state);
        super::StagedPhysicalRecovery { state: self }
    }

    pub(super) const fn state(&self) -> &StagedRecoveryState {
        &self.0.as_slice()[0]
    }

    pub(super) fn into_state(mut self) -> StagedRecoveryState {
        self.0
            .pop()
            .expect("closed staged owner contains its admitted state")
    }
}
