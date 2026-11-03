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
        for extraction in [0, payload] {
            let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
            let movement = (payload + extraction).max(
                payload
                    + std::mem::size_of::<crate::orchestration::RecoveryStagingInput>() as u64
                    + 2 * staged,
            );
            let required = 17 + payload + staged + movement;
            let mut allowance = PlanningResidentAllowance::new(17, required - 1).unwrap();
            assert!(matches!(
                PlannedStateBacking::prepare(&mut allowance, extraction),
                Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) if observed == required
            ));
            assert_eq!(allowance.used(), 17);
            assert_eq!(allowance.peak(), 17);
        }
    }

    #[test]
    fn planned_backing_peak_keeps_actual_capacity_and_construction_overlap() {
        let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
        for extraction in [0, payload] {
            let mut allowance = PlanningResidentAllowance::new(17, u64::MAX).unwrap();
            let backing = PlannedStateBacking::prepare(&mut allowance, extraction).unwrap();
            let planned_backing = PlanningResidentAllowance::vector_bytes(&backing.values).unwrap();
            let staged_backing = backing.staged.backing_bytes().unwrap();
            let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
            let movement = (payload + extraction).max(
                payload
                    + std::mem::size_of::<crate::orchestration::RecoveryStagingInput>() as u64
                    + 2 * staged,
            );
            let actual = planned_backing + staged_backing;
            assert!(planned_backing >= payload);
            assert!(staged_backing >= staged);
            assert_eq!(allowance.used(), 17 + actual);
            assert_eq!(allowance.peak(), 17 + actual + movement);
            assert!(backing.values.is_empty());
            drop(backing);
            allowance.release(actual);
            assert_eq!(allowance.used(), 17);
            assert_eq!(allowance.peak(), 17 + actual + movement);
        }
    }
}

/// One admitted payload slot; no caller can publish an empty planned receiver.
pub(crate) struct PlannedStateBacking {
    values: Vec<PlannedRecoveryState>,
    staged: crate::progression::StagedStateBacking,
}

impl PlannedStateBacking {
    pub(crate) fn prepare(
        allowance: &mut super::PlanningResidentAllowance,
        extraction_bytes: u64,
    ) -> Result<Self, super::PlanningMemoryDenial> {
        let payload = super::PlanningResidentAllowance::slot_bytes::<PlannedRecoveryState>(1)?;
        let staged = super::PlanningResidentAllowance::slot_bytes::<
            crate::progression::StagedRecoveryState,
        >(1)?;
        let overflow = || super::PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX };
        let scratch = construction_scratch(extraction_bytes).ok_or_else(overflow)?;
        let backing = payload.checked_add(staged).ok_or_else(overflow)?;
        allowance.transient(backing.checked_add(scratch).ok_or_else(overflow)?)?;
        let values = allowance.reserve::<PlannedRecoveryState>(1)?;
        let staged = crate::progression::StagedStateBacking::prepare(allowance)?;
        allowance.transient(scratch)?;
        Ok(Self { values, staged })
    }

    pub(super) fn finish(mut self, state: PlannedRecoveryState) -> super::PlannedPhysicalRecovery {
        assert!(self.values.is_empty() && self.values.capacity() >= 1);
        self.values.push(state);
        super::PlannedPhysicalRecovery { state: self }
    }

    pub(super) const fn state(&self) -> &PlannedRecoveryState {
        &self.values.as_slice()[0]
    }

    pub(super) fn into_state(mut self) -> PlannedRecoveryState {
        self.values
            .pop()
            .expect("planned receiver contains its admitted state")
    }

    pub(super) fn into_staging_parts(
        mut self,
    ) -> (PlannedRecoveryState, crate::progression::StagedStateBacking) {
        let state = self
            .values
            .pop()
            .expect("planned receiver contains its admitted state");
        (state, self.staged)
    }
}

fn construction_scratch(extraction_bytes: u64) -> Option<u64> {
    let planned = std::mem::size_of::<PlannedRecoveryState>() as u64;
    let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
    let staging_input = std::mem::size_of::<crate::orchestration::RecoveryStagingInput>() as u64;
    Some(
        planned.checked_add(extraction_bytes)?.max(
            planned
                .checked_add(staging_input)?
                .checked_add(2 * staged)?,
        ),
    )
}
