use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, RecoveryPlanCost, RecoveryPlanningCounters,
};

use crate::entry::{AdmittedPlatformAuthority, PhysicalRecoverySourceDenial};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::{RecoveryCoordination, ResidentSourceSelection};

use super::{
    PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence, RecoveryPublicationPlan,
    RecoveryQuiescencePlan, RecoveryStagingLayoutPlan,
};

pub(super) struct PlannedRecoveryState {
    pub(super) authority: AdmittedPlatformAuthority,
    pub(super) coordination: RecoveryCoordination,
    pub(super) selection: ResidentSourceSelection,
    pub(super) custody: crate::progression::PlanningCustody,
    pub(super) verified_selected_tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(super) discovery_counters: PhysicalRecoveryDiscoveryCounters,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) integrity: RecoveryIntegrityEvidence,
    pub(super) freshness: StoreRecoveryBindingFreshnessSample,
    pub(super) fates: RecoveryOperationFateSet,
    pub(super) redo: ImmutablePhysicalRedoPlan,
    pub(super) plan_cost: RecoveryPlanCost,
    pub(super) planning_counters: RecoveryPlanningCounters,
    pub(super) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(super) staging: RecoveryStagingLayoutPlan,
    pub(super) publication: RecoveryPublicationPlan,
    pub(super) quiescence: RecoveryQuiescencePlan,
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

#[cfg(test)]
mod tests {
    use super::super::{PlanningMemoryDenial, PlanningResidentAllowance};
    use super::*;

    #[test]
    fn planned_backing_refuses_combined_construction_before_allocating() {
        let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
        let mut allowance = PlanningResidentAllowance::new(17, 17 + 2 * payload - 1).unwrap();
        assert!(matches!(
            PlannedStateBacking::prepare(&mut allowance),
            Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed })
                if observed == 17 + 2 * payload
        ));
        assert_eq!(allowance.used(), 17);
        assert_eq!(allowance.peak(), 17);
    }

    #[test]
    fn planned_backing_peak_keeps_actual_capacity_and_construction_overlap() {
        let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
        let mut allowance = PlanningResidentAllowance::new(17, u64::MAX).unwrap();
        let backing = PlannedStateBacking::prepare(&mut allowance).unwrap();
        let actual = PlanningResidentAllowance::vector_bytes(&backing.0).unwrap();
        assert!(actual >= payload);
        assert_eq!(allowance.used(), 17 + actual);
        assert_eq!(allowance.peak(), 17 + actual + payload);
        assert!(backing.0.is_empty());
        drop(backing);
        allowance.release(actual);
        assert_eq!(allowance.used(), 17);
        assert_eq!(allowance.peak(), 17 + actual + payload);
    }
}

/// One admitted payload slot; no caller can publish an empty planned receiver.
pub(crate) struct PlannedStateBacking(Vec<PlannedRecoveryState>);

impl PlannedStateBacking {
    pub(crate) fn prepare(
        allowance: &mut super::PlanningResidentAllowance,
    ) -> Result<Self, super::PlanningMemoryDenial> {
        // Admit backing and construction overlap together, before allocation.
        allowance.transient(super::PlanningResidentAllowance::slot_bytes::<
            PlannedRecoveryState,
        >(2)?)?;
        let values = allowance.reserve::<PlannedRecoveryState>(1)?;
        allowance.transient(super::PlanningResidentAllowance::slot_bytes::<
            PlannedRecoveryState,
        >(1)?)?;
        Ok(Self(values))
    }

    pub(super) fn finish(mut self, state: PlannedRecoveryState) -> super::PlannedPhysicalRecovery {
        assert!(self.0.is_empty() && self.0.capacity() >= 1);
        self.0.push(state);
        super::PlannedPhysicalRecovery { state: self }
    }

    pub(super) const fn state(&self) -> &PlannedRecoveryState {
        &self.0.as_slice()[0]
    }

    pub(super) fn into_state(mut self) -> PlannedRecoveryState {
        self.0
            .pop()
            .expect("planned receiver contains its admitted state")
    }
}
